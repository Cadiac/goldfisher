use log::{warn};
use std::rc::Rc;

use crate::card::{CardRef, CardType, SearchFilter, Zone};
use crate::game::Game;
use crate::strategy::Strategy;
use crate::utils::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    Mill(usize),
    Draw(usize),
    UntapLands(Option<usize>),
    DamageEach(i32),
    SearchAndPutHand(Option<SearchFilter>),
    SearchAndPutTopOfLibrary(Option<SearchFilter>),
    SearchAndPutBattlefield(Option<SearchFilter>),
    Impulse(usize),
    Intuition,
    CavernHarpy,
    Unearth,
    WordsOfWisdom,
    Snap,
    FranticSearch,
    BrainFreeze,
    Meditate,
    Brainstorm,
    Ponder,
    Preordain,
}

impl Effect {
    /// Resolves the given effect, applying its effect to the game.
    pub fn resolve(&self, game: &mut Game, source: &CardRef, strategy: &impl Strategy) {
        match self {
            Effect::SearchAndPutHand(search_filter) => {
                self.search_hand(game, source, strategy, search_filter)
            },
            Effect::SearchAndPutTopOfLibrary(search_filter) => {
                self.search_top_of_library(game, source, strategy, search_filter)
            },
            Effect::Impulse(amount) => self.impulse(game, source, strategy, *amount),
            Effect::Intuition => self.intuition(game, source, strategy),
            Effect::CavernHarpy => self.cavern_harpy(game, source, strategy),
            Effect::Unearth => self.unearth(game, source, strategy),
            Effect::UntapLands(amount) => self.untap_lands(game, source, strategy, *amount),
            Effect::DamageEach(amount) => self.damage_each(game, source, strategy, *amount),
            Effect::WordsOfWisdom => {
                game.draw_n(2);
                game.opponent_library -= 1;
            },
            Effect::Snap => {
                // TODO: Make this target
                let cloud_of_faeries_to_return = game.game_objects.iter().find(|card| {
                    let card = card.borrow();
                    card.zone == Zone::Battlefield && card.name == "Cloud of Faeries"
                });
        
                if let Some(card) = cloud_of_faeries_to_return {
                    game.log(format!(
                        "[Turn {turn:002}][Action]: Bouncing \"Cloud of Faeries\" back to hand.",
                        turn = game.turn
                    ));
                    card.borrow_mut().zone = Zone::Hand;
                }

                self.untap_lands(game, source, strategy, Some(2));
            },
            Effect::FranticSearch => {
                let hand_size = game.game_objects.iter().filter(is_hand).count();
                game.draw_n(2);
                let cards_to_discard = strategy.discard_to_hand_size(game, hand_size);
                for card in cards_to_discard {
                    game.discard(card);
                }
                self.untap_lands(game, source, strategy, Some(3));
            },
            Effect::Meditate => {
                game.draw_n(4);
                game.turns_to_skip += 1;
            },
            Effect::Mill(amount) => {
                game.opponent_library -= *amount as i32;
            },
            Effect::Draw(amount) => {
                game.draw_n(*amount);
            },
            Effect::BrainFreeze => {
                // TODO: Make this target
                let cards_to_mill = 3 * game.storm as i32;

                game.log(format!(
                    "[Turn {turn:002}][Action]: Brain Freeze with Storm {storm}: milling opponent for {cards_to_mill}",
                    turn = game.turn,
                    storm = game.storm,
                ));

                game.opponent_library -= cards_to_mill;
            },
            Effect::Brainstorm => {
                // At this time the card is on graveyard already
                let hand_size = game.game_objects.iter().filter(is_hand).count();
                game.draw_n(3);
                let cards_to_discard = strategy.discard_to_hand_size(game, hand_size + 1);
                for card in cards_to_discard {
                    card.borrow_mut().zone = Zone::Library;
                    game.deck.put_top(card);
                }
            },
            Effect::Ponder => {
                // TODO: actual ponder
                self.impulse(game, source, strategy, 3)
            },
            Effect::Preordain => {
                // TODO: actual Preordain
                self.impulse(game, source, strategy, 2)
            },
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
            game.log(format!("[Turn {turn:002}][Action]: Searched for \"{card_name}\" and put it on top of the library.",
                turn = game.turn,
                card_name = found.borrow().name));

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
            if let Some(SearchFilter::Wish(_card_types)) = search_filter {
                game.log(format!("[Turn {turn:002}][Action]: Searched for \"{card_name}\" from sideboard and put it in hand.",
                            turn = game.turn,
                            card_name = found.borrow().name));

                game.deck.remove_sideboard(&found);
                found.borrow_mut().zone = Zone::Hand;
                game.game_objects.push(found);
            } else {
                game.log(format!(
                    "[Turn {turn:002}][Action]: Searched for \"{card_name}\" and put it in hand.",
                    turn = game.turn,
                    card_name = found.borrow().name
                ));

                game.deck.remove(&found);
                found.borrow_mut().zone = Zone::Hand;
                game.deck.shuffle();
            }
        } else {
            game.log(format!(
                "[Turn {turn:002}][Action]: Failed to find.",
                turn = game.turn
            ));
        }

        if let Some(SearchFilter::Wish(_card_types)) = search_filter {
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
        game.log(format!(
            "[Turn {turn:002}][Action]: Looking at cards: {revealed_str}",
            turn = game.turn
        ));

        if let Some(selected) = strategy.select_best(game, group_by_name(cards.clone())) {
            game.log(format!(
                "[Turn {turn:002}][Action]: Selected \"{card_name}\" and put it in hand.",
                turn = game.turn,
                card_name = selected.borrow().name
            ));
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
            game.log(format!(
                "[Turn {turn:002}][Action]: Returning \"{card_name}\" on the battlefield.",
                turn = game.turn,
                card_name = target.borrow().name
            ));
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
                    && card.card_types.contains(&CardType::Creature)
                    && card.cost.values().sum::<i32>() <= 3
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
        lands_to_untap: Option<usize>,
    ) {
        let untap = 
            match lands_to_untap {
                Some(count) => count,
                None => {
                    // Untap all lands
                    game
                        .game_objects
                        .iter()
                        .filter(|card| {
                            is_battlefield(card) && is_card_type(card, &CardType::Land) && is_tapped(card)
                        })
                        .count()
                }
            };

        for _ in 0..untap {
            let mut tapped_lands = game
                .game_objects
                .iter()
                .filter(|card| {
                    is_battlefield(card) && is_card_type(card, &CardType::Land) && is_tapped(card)
                })
                .cloned()
                .collect::<Vec<_>>();

            tapped_lands.sort_by(sort_by_best_mana_to_play);

            if let Some(card) = tapped_lands.last() {
                game.log(format!(
                    "[Turn {turn:002}][Action]: Untapping \"{card_name}\".",
                    card_name = card.borrow().name,
                    turn = game.turn
                ));
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
            game.log(format!(
                "[Turn {turn:002}][Action]: Bouncing \"Maggot Carrier\" back to hand.",
                turn = game.turn
            ));
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
            game.log(format!(
                "[Turn {turn:002}][Action]: Bouncing \"Cavern Harpy\" back to hand.",
                turn = game.turn
            ));
            source.borrow_mut().zone = Zone::Hand;
            return;
        }

        let cloud_of_faeries_to_return = game.game_objects.iter().find(|card| {
            let card = card.borrow();
            card.zone == Zone::Battlefield && card.name == "Cloud of Faeries"
        });

        if let Some(card) = cloud_of_faeries_to_return {
            game.log(format!(
                "[Turn {turn:002}][Action]: Bouncing \"Cloud of Faeries\" back to hand.",
                turn = game.turn
            ));
            card.borrow_mut().zone = Zone::Hand;
            return;
        }

        let raven_familiar_to_return = game.game_objects.iter().find(|card| {
            let card = card.borrow();
            card.zone == Zone::Battlefield && card.name == "Raven Familiar"
        });

        if let Some(card) = raven_familiar_to_return {
            game.log(format!(
                "[Turn {turn:002}][Action]: Bouncing \"Raven Familiar\" back to hand.",
                turn = game.turn
            ));
            card.borrow_mut().zone = Zone::Hand;
            return;
        }

        // Otherwise we must bounce the Harpy back to hand
        game.log(format!(
            "[Turn {turn:002}][Action]: Bouncing \"Cavern Harpy\" back to hand.",
            turn = game.turn
        ));
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

        game.log(format!(
            "[Turn {turn:002}][Action]: Searched for cards: {found_str} with Intuition.",
            turn = game.turn
        ));

        if let Some(card) = found.pop() {
            game.deck.remove(&card);
            card.borrow_mut().zone = Zone::Hand;

            game.log(format!(
                "[Turn {turn:002}][Action]: Put \"{card_name}\" to hand.",
                card_name = card.borrow().name,
                turn = game.turn
            ));
        }

        for card in found.into_iter() {
            game.deck.remove(&card);
            card.borrow_mut().zone = Zone::Graveyard;

            game.log(format!(
                "[Turn {turn:002}][Action]: Put \"{card_name}\" to graveyard.",
                card_name = card.borrow().name,
                turn = game.turn
            ));
        }
    }
}
