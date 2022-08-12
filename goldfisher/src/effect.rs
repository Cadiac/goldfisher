use log::{debug, warn};
use std::rc::Rc;

use crate::card::{CardRef, CardType, SearchFilter, Zone};
use crate::game::Game;
use crate::strategy::Strategy;
use crate::utils::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    SearchAndPutHand(Option<SearchFilter>),
    SearchAndPutTopOfLibrary(Option<SearchFilter>),
    SearchAndPutBattlefield(Option<SearchFilter>),
    Impulse(usize),
    Intuition,
    CavernHarpy,
    Unearth,
    UntapLands(usize),
    DamageEach(i32),
}

impl Effect {
    /// Resolves the given effect, applying its effect to the game.
    pub fn resolve(&self, game: &mut Game, source: &CardRef, strategy: &impl Strategy) {
        match self {
            Effect::SearchAndPutHand(search_filter) => {
                self.search_hand(game, source, strategy, search_filter)
            }
            Effect::SearchAndPutTopOfLibrary(search_filter) => {
                self.search_top_of_library(game, source, strategy, search_filter)
            }
            Effect::Impulse(amount) => self.impulse(game, source, strategy, *amount),
            Effect::Intuition => self.intuition(game, source, strategy),
            Effect::CavernHarpy => self.cavern_harpy(game, source, strategy),
            Effect::Unearth => self.unearth(game, source, strategy),
            Effect::UntapLands(amount) => self.untap_lands(game, source, strategy, *amount),
            Effect::DamageEach(amount) => self.damage_each(game, source, strategy, *amount),
            _ => unimplemented!(),
        }
    }

    fn search_top_of_library(
        &self,
        game: &mut Game,
        _source: &CardRef,
        strategy: &impl Strategy,
        search_filter: &Option<SearchFilter>,
    ) {
        let searchable = apply_search_filter(game, search_filter);
        if let Some(found) = strategy.select_best(game, group_by_name(searchable)) {
            debug!("[Turn {turn:002}][Action]: Searched for \"{card_name}\" and put it on top of the library.",
                turn = game.turn,
                card_name = found.borrow().name);

            game.deck.remove(&found);
            game.deck.shuffle();
            game.deck.put_top(found);
        }
    }

    fn search_hand(
        &self,
        game: &mut Game,
        source: &CardRef,
        strategy: &impl Strategy,
        search_filter: &Option<SearchFilter>,
    ) {
        let searchable = apply_search_filter(game, search_filter);
        if let Some(found) = strategy.select_best(game, group_by_name(searchable)) {
            if *search_filter == Some(SearchFilter::LivingWish) {
                debug!("[Turn {turn:002}][Action]: Searched for \"{card_name}\" from sideboard and put it in hand.",
                            turn = game.turn,
                            card_name = found.borrow().name);

                game.deck.remove_sideboard(&found);
                found.borrow_mut().zone = Zone::Hand;
                game.game_objects.push(found);
            } else {
                debug!(
                    "[Turn {turn:002}][Action]: Searched for \"{card_name}\" and put it in hand.",
                    turn = game.turn,
                    card_name = found.borrow().name
                );

                game.deck.remove(&found);
                found.borrow_mut().zone = Zone::Hand;
                game.deck.shuffle();
            }
        } else {
            debug!(
                "[Turn {turn:002}][Action]: Failed to find.",
                turn = game.turn
            );
        }

        if *search_filter == Some(SearchFilter::LivingWish) {
            source.borrow_mut().zone = Zone::Exile;
        }
    }

    fn impulse(
        &self,
        game: &mut Game,
        _source: &CardRef,
        strategy: &impl Strategy,
        amount_to_look_at: usize,
    ) {
        let mut cards = Vec::with_capacity(amount_to_look_at);
        for _ in 0..amount_to_look_at {
            // This isn't actually "draw"
            if let Some(card) = game.deck.draw() {
                if card.borrow().zone != Zone::Library {
                    warn!(
                        "Card {} is on the wrong zone {:?}!",
                        card.borrow().name,
                        card.borrow().zone
                    );
                    panic!("wrong zone");
                }
                cards.push(card);
            }
        }

        let revealed_str = cards
            .iter()
            .map(|card| format!("\"{}\"", card.borrow().name.clone()))
            .collect::<Vec<_>>()
            .join(", ");
        debug!(
            "[Turn {turn:002}][Action]: Looking at cards: {revealed_str}",
            turn = game.turn
        );

        if let Some(selected) = strategy.select_best(game, group_by_name(cards.clone())) {
            debug!(
                "[Turn {turn:002}][Action]: Selected \"{card_name}\" and put it in hand.",
                turn = game.turn,
                card_name = selected.borrow().name
            );
            cards.retain(|card| !Rc::ptr_eq(card, &selected));

            selected.borrow_mut().zone = Zone::Hand;
        }

        for card in cards {
            game.deck.put_bottom(card);
        }
    }

    fn reanimate(
        &self,
        game: &mut Game,
        _source: &CardRef,
        strategy: &impl Strategy,
        possible_targets: Vec<CardRef>,
    ) {
        if let Some(target) = strategy.select_best(game, group_by_name(possible_targets)) {
            debug!(
                "[Turn {turn:002}][Action]: Returning \"{card_name}\" on the battlefield.",
                turn = game.turn,
                card_name = target.borrow().name
            );
            target.borrow_mut().zone = Zone::Battlefield;
            game.handle_on_resolve_effects(&target, strategy)
        }
    }

    fn unearth(&self, game: &mut Game, source: &CardRef, strategy: &impl Strategy) {
        let possible_targets = game
            .game_objects
            .iter()
            .filter(|card| {
                let card = card.borrow();
                card.zone == Zone::Graveyard
                    && card.card_type == CardType::Creature
                    && card.cost.values().sum::<usize>() <= 3
            })
            .cloned()
            .collect();

        self.reanimate(game, source, strategy, possible_targets);
    }

    fn untap_lands(
        &self,
        game: &mut Game,
        _source: &CardRef,
        _strategy: &impl Strategy,
        lands_to_untap: usize,
    ) {
        for _ in 0..lands_to_untap {
            let mut tapped_lands = game
                .game_objects
                .iter()
                .filter(|card| {
                    is_battlefield(card) && is_card_type(card, CardType::Land) && is_tapped(card)
                })
                .cloned()
                .collect::<Vec<_>>();

            tapped_lands.sort_by(sort_by_best_mana_to_play);

            if let Some(card) = tapped_lands.last() {
                debug!(
                    "[Turn {turn:002}][Action]: Untapping \"{card_name}\".",
                    card_name = card.borrow().name,
                    turn = game.turn
                );
                card.borrow_mut().is_tapped = false;
            }
        }
    }

    fn cavern_harpy(&self, game: &mut Game, source: &CardRef, _strategy: &impl Strategy) {
        let maggot_carrier_to_return = game.game_objects.iter().find(|card| {
            let card = card.borrow();
            card.zone == Zone::Battlefield && card.name == "Maggot Carrier"
        });

        if let Some(card) = maggot_carrier_to_return {
            debug!(
                "[Turn {turn:002}][Action]: Bouncing \"Maggot Carrier\" back to hand.",
                turn = game.turn
            );
            card.borrow_mut().zone = Zone::Hand;
            return;
        }

        let etb_draw_triggers = game
            .game_objects
            .iter()
            .filter(|card| {
                let card = card.borrow();
                card.zone == Zone::Battlefield && card.name == "Wirewood Savage"
            })
            .count();

        if etb_draw_triggers > 0 && game.deck.len() > 1 {
            debug!(
                "[Turn {turn:002}][Action]: Bouncing \"Cavern Harpy\" back to hand.",
                turn = game.turn
            );
            source.borrow_mut().zone = Zone::Hand;
            return;
        }

        let cloud_of_faeries_to_return = game.game_objects.iter().find(|card| {
            let card = card.borrow();
            card.zone == Zone::Battlefield && card.name == "Cloud of Faeries"
        });

        if let Some(card) = cloud_of_faeries_to_return {
            debug!(
                "[Turn {turn:002}][Action]: Bouncing \"Cloud of Faeries\" back to hand.",
                turn = game.turn
            );
            card.borrow_mut().zone = Zone::Hand;
            return;
        }

        let raven_familiar_to_return = game.game_objects.iter().find(|card| {
            let card = card.borrow();
            card.zone == Zone::Battlefield && card.name == "Raven Familiar"
        });

        if let Some(card) = raven_familiar_to_return {
            debug!(
                "[Turn {turn:002}][Action]: Bouncing \"Raven Familiar\" back to hand.",
                turn = game.turn
            );
            card.borrow_mut().zone = Zone::Hand;
            return;
        }

        // Otherwise we must bounce the Harpy back to hand
        debug!(
            "[Turn {turn:002}][Action]: Bouncing \"Cavern Harpy\" back to hand.",
            turn = game.turn
        );
        source.borrow_mut().zone = Zone::Hand;
    }

    fn damage_each(
        &self,
        game: &mut Game,
        _source: &CardRef,
        _strategy: &impl Strategy,
        damage: i32,
    ) {
        game.damage_each(damage as i32);
    }

    fn intuition(&self, game: &mut Game, _source: &CardRef, strategy: &impl Strategy) {
        let mut found = strategy.select_intuition(game);
        let found_str = found
            .iter()
            .map(|card| format!("\"{}\"", card.borrow().name))
            .collect::<Vec<_>>()
            .join(", ");

        debug!(
            "[Turn {turn:002}][Action]: Searched for cards: {found_str} with Intuition.",
            turn = game.turn
        );

        if let Some(card) = found.pop() {
            game.deck.remove(&card);
            card.borrow_mut().zone = Zone::Hand;

            debug!(
                "[Turn {turn:002}][Action]: Put \"{card_name}\" to hand.",
                card_name = card.borrow().name,
                turn = game.turn
            );
        }

        for card in found.into_iter() {
            game.deck.remove(&card);
            card.borrow_mut().zone = Zone::Graveyard;

            debug!(
                "[Turn {turn:002}][Action]: Put \"{card_name}\" to graveyard.",
                card_name = card.borrow().name,
                turn = game.turn
            );
        }
    }
}
