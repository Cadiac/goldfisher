use log::{debug, warn};
use std::collections::HashMap;
use std::rc::Rc;

use crate::card::{CardRef, CardType, Effect, SearchFilter, SubType, Zone};
use crate::deck::{Deck, Decklist};
use crate::mana::find_payment_for;
use crate::mana::{Mana, PaymentAndFloating};
use crate::strategy::Strategy;
use crate::utils::*;

pub enum GameStatus {
    Continue,
    Draw(usize),
    Win(usize),
    Lose(usize),
}

pub struct GameState {
    pub turn: usize,
    pub game_objects: Vec<CardRef>,
    pub available_land_drops: usize,
    pub deck: Deck,
    pub life_total: i32,
    pub damage_dealt: i32,
    pub floating_mana: HashMap<Mana, usize>,
    pub is_first_player: bool,
}

impl GameState {
    pub fn new(decklist: Decklist) -> Self {
        let mut deck: Deck = decklist.into();

        let mut game_objects = Vec::with_capacity(deck.len());
        for card in deck.iter() {
            game_objects.push(card.clone())
        }

        debug!("Deck size: {deck_size}", deck_size = deck.len());

        deck.shuffle();

        Self {
            deck,
            game_objects,
            turn: 0,
            life_total: 20,
            damage_dealt: 0,
            floating_mana: HashMap::new(),
            is_first_player: true,
            available_land_drops: 1,
        }
    }

    pub fn find_castable(&self) -> Vec<(CardRef, Option<PaymentAndFloating>)> {
        let nonlands_in_hand = self.game_objects.iter().filter(|card| {
            let card = card.borrow();
            card.zone == Zone::Hand && card.card_type != CardType::Land
        });

        let mut mana_sources: Vec<_> = self
            .game_objects
            .iter()
            .filter(|card| {
                let card = card.borrow();
                let is_untapped_mana_source = card.zone == Zone::Battlefield
                    && !card.produced_mana.is_empty()
                    && !card.is_summoning_sick
                    && !card.is_tapped
                    && card.name != "Elvish Spirit Guide";

                if is_untapped_mana_source {
                    return true;
                }

                return card.name == "Elvish Spirit Guide" && card.zone == Zone::Hand;
            })
            .map(Rc::clone)
            .collect();

        mana_sources.sort_by(sort_by_best_mana_to_use);

        let is_aluren_active = self
            .game_objects
            .iter()
            .any(|card| is_battlefield(&card) && card.borrow().name == "Aluren");

        let castable = nonlands_in_hand
            .map(|card| {
                (
                    card.clone(),
                    find_payment_for(
                        card.clone(),
                        &mana_sources,
                        self.floating_mana.clone(),
                        is_aluren_active,
                    ),
                )
            })
            .filter(|(_, payment)| payment.is_some());

        castable.collect()
    }

    pub fn play_land(&mut self, land_card: CardRef) {
        if self.available_land_drops > 0 {
            self.available_land_drops -= 1;
            let mut card = land_card.borrow_mut();

            debug!(
                "[Turn {turn:002}][Action]: Playing land: \"{name}\"",
                turn = self.turn,
                name = card.name
            );

            card.zone = Zone::Battlefield;
        }
    }

    pub fn draw_n(&mut self, amount: usize) {
        for _ in 0..amount {
            self.draw();
        }
    }

    pub fn draw(&mut self) -> GameStatus {
        if self.turn == 0 || (self.turn == 1 && !self.is_first_player) || self.turn > 1 {
            if let Some(card) = self.deck.draw() {
                let mut card = card.borrow_mut();

                card.zone = Zone::Hand;
                debug!(
                    "[Turn {turn:002}][Action]: Drew card: \"{name}\", {library} cards remaining.",
                    turn = self.turn,
                    name = card.name,
                    library = self.deck.len(),
                );
                return GameStatus::Continue;
            } else {
                return GameStatus::Lose(self.turn);
            }
        }
        GameStatus::Continue
    }

    pub fn untap(&mut self) {
        for card in self.game_objects.iter() {
            let mut card = card.borrow_mut();

            if card.zone == Zone::Battlefield {
                card.is_summoning_sick = false;
                card.is_tapped = false;
            }
        }
    }

    pub fn take_game_actions(&mut self, strategy: &Box<dyn Strategy>) -> GameStatus {
        loop {
            let action_taken = strategy.take_game_action(self);
            match strategy.game_status(self) {
                GameStatus::Continue => {
                    if !action_taken {
                        return GameStatus::Continue;
                    }
                }
                result => return result,
            };
        }
    }

    pub fn cast_spell(
        &mut self,
        strategy: &impl Strategy,
        source: &CardRef,
        (payment, floating): &PaymentAndFloating,
        attach_to: Option<CardRef>,
    ) {
        let target_str = match attach_to.as_ref() {
            Some(target) => format!(" on target \"{}\"", target.borrow().name.clone()),
            None => "".to_owned(),
        };

        let mana_sources_str = if payment.is_empty() {
            String::new()
        } else {
            let mana_sources = payment
                .iter()
                .map(|mana_source| format!("\"{}\"", mana_source.borrow().name.clone()))
                .collect::<Vec<_>>()
                .join(", ");
            format!(" with mana sources: {mana_sources}")
        };

        debug!("[Turn {turn:002}][Action]: Casting card: \"{card_name}\"{target_str}{mana_sources_str}",
            turn = self.turn,
            card_name = source.borrow().name);

        let new_zone = match source.borrow().card_type {
            CardType::Creature | CardType::Enchantment | CardType::Land | CardType::Artifact => {
                Zone::Battlefield
            }
            CardType::Sorcery | CardType::Instant => Zone::Graveyard,
        };

        source.borrow_mut().zone = new_zone;
        source.borrow_mut().attached_to = attach_to;

        if source.borrow().card_type == CardType::Creature {
            source.borrow_mut().is_summoning_sick = true;

            if source.borrow().sub_types.contains(&SubType::Beast) {
                let etb_draw_triggers = self
                    .game_objects
                    .iter()
                    .filter(|card| {
                        let card = card.borrow();
                        card.zone == Zone::Battlefield && card.name == "Wirewood Savage"
                    })
                    .count();

                for _ in 0..etb_draw_triggers {
                    // Leave one card so that turn can be passed
                    if self.deck.len() > 1 {
                        self.draw();
                    }
                }
            }

            let lifegain_triggers = self
                .game_objects
                .iter()
                .filter(|card| {
                    let card = card.borrow();
                    card.zone == Zone::Battlefield && card.name == "Soul Warden"
                })
                .count();

            for _ in 0..lifegain_triggers {
                self.take_damage(-1);
            }
        }

        for mana_source in payment {
            let mut source = mana_source.borrow_mut();

            if let Some(uses) = source.remaining_uses {
                if uses > 1 {
                    source.remaining_uses = Some(uses - 1);
                    source.is_tapped = true;
                } else {
                    source.remaining_uses = Some(0);
                    if source.name == "Elvish Spirit Guide" {
                        source.zone = Zone::Exile;
                    } else {
                        source.zone = Zone::Graveyard;
                    }
                }
            } else {
                source.is_tapped = true;
            }
        }

        self.floating_mana = floating.to_owned();

        self.handle_on_resolve_effects(source, strategy);
    }

    fn handle_on_resolve_effects(&mut self, game_object: &CardRef, strategy: &impl Strategy) {
        let effect = match game_object.borrow().on_resolve.clone() {
            Some(e) => e,
            _ => return,
        };

        match effect {
            Effect::SearchAndPutTopOfLibrary(search_filter) => {
                let searchable = apply_search_filter(self, search_filter);
                if let Some(found) = strategy.select_best(self, group_by_name(searchable)) {
                    debug!("[Turn {turn:002}][Action]: Searched for \"{card_name}\" and put it on top of the library.",
                        turn = self.turn,
                        card_name = found.borrow().name);

                    self.deck.remove(&found);
                    self.deck.shuffle();
                    self.deck.put_top(found);
                }
            }
            Effect::SearchAndPutHand(search_filter) => {
                let searchable = apply_search_filter(self, search_filter.clone());
                if let Some(found) = strategy.select_best(self, group_by_name(searchable)) {
                    if search_filter == Some(SearchFilter::LivingWish) {
                        debug!("[Turn {turn:002}][Action]: Searched for \"{card_name}\" from sideboard and put it in hand.",
                                    turn = self.turn,
                                    card_name = found.borrow().name);

                        self.deck.remove_sideboard(&found);
                        found.borrow_mut().zone = Zone::Hand;
                        self.game_objects.push(found);
                    } else {
                        debug!("[Turn {turn:002}][Action]: Searched for \"{card_name}\" and put it in hand.",
                            turn = self.turn,
                            card_name = found.borrow().name);

                        self.deck.remove(&found);
                        found.borrow_mut().zone = Zone::Hand;
                        self.deck.shuffle();
                    }
                } else {
                    debug!(
                        "[Turn {turn:002}][Action]: Failed to find.",
                        turn = self.turn
                    );
                }

                if search_filter == Some(SearchFilter::LivingWish) {
                    game_object.borrow_mut().zone = Zone::Exile;
                }
            }
            Effect::Impulse(n) => {
                let mut cards = Vec::with_capacity(n);
                for _ in 0..n {
                    // This isn't actually "draw"
                    if let Some(card) = self.deck.draw() {
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
                    turn = self.turn
                );

                if let Some(selected) = strategy.select_best(self, group_by_name(cards.clone())) {
                    debug!(
                        "[Turn {turn:002}][Action]: Selected \"{card_name}\" and put it in hand.",
                        turn = self.turn,
                        card_name = selected.borrow().name
                    );
                    cards.retain(|card| !Rc::ptr_eq(card, &selected));

                    selected.borrow_mut().zone = Zone::Hand;
                }

                for card in cards {
                    self.deck.put_bottom(card);
                }
            }
            Effect::Unearth => {
                let graveyard = self
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

                if let Some(target) = strategy.select_best(self, group_by_name(graveyard)) {
                    debug!(
                        "[Turn {turn:002}][Action]: Returning \"{card_name}\" on the battlefield.",
                        turn = self.turn,
                        card_name = target.borrow().name
                    );
                    target.borrow_mut().zone = Zone::Battlefield;
                    self.handle_on_resolve_effects(&target, strategy)
                }
            }
            Effect::UntapLands(n) => {
                for _ in 0..n {
                    let mut tapped_lands = self
                        .game_objects
                        .iter()
                        .filter(|card| {
                            is_battlefield(card)
                                && is_card_type(card, CardType::Land)
                                && is_tapped(card)
                        })
                        .cloned()
                        .collect::<Vec<_>>();

                    tapped_lands.sort_by(sort_by_best_mana_to_play);

                    if let Some(card) = tapped_lands.last() {
                        debug!(
                            "[Turn {turn:002}][Action]: Untapping \"{card_name}\".",
                            card_name = card.borrow().name,
                            turn = self.turn
                        );
                        card.borrow_mut().is_tapped = false;
                    }
                }
            }
            Effect::CavernHarpy => {
                let maggot_carrier_to_return = self.game_objects.iter().find(|card| {
                    let card = card.borrow();
                    card.zone == Zone::Battlefield && card.name == "Maggot Carrier"
                });

                if let Some(card) = maggot_carrier_to_return {
                    debug!(
                        "[Turn {turn:002}][Action]: Bouncing \"Maggot Carrier\" back to hand.",
                        turn = self.turn
                    );
                    card.borrow_mut().zone = Zone::Hand;
                    return;
                }

                let etb_draw_triggers = self
                    .game_objects
                    .iter()
                    .filter(|card| {
                        let card = card.borrow();
                        card.zone == Zone::Battlefield && card.name == "Wirewood Savage"
                    })
                    .count();

                if etb_draw_triggers > 0 && self.deck.len() > 1 {
                    debug!(
                        "[Turn {turn:002}][Action]: Bouncing \"Cavern Harpy\" back to hand.",
                        turn = self.turn
                    );
                    game_object.borrow_mut().zone = Zone::Hand;
                    return;
                }

                let cloud_of_faeries_to_return = self.game_objects.iter().find(|card| {
                    let card = card.borrow();
                    card.zone == Zone::Battlefield && card.name == "Cloud of Faeries"
                });

                if let Some(card) = cloud_of_faeries_to_return {
                    debug!(
                        "[Turn {turn:002}][Action]: Bouncing \"Cloud of Faeries\" back to hand.",
                        turn = self.turn
                    );
                    card.borrow_mut().zone = Zone::Hand;
                    return;
                }

                let raven_familiar_to_return = self.game_objects.iter().find(|card| {
                    let card = card.borrow();
                    card.zone == Zone::Battlefield && card.name == "Raven Familiar"
                });

                if let Some(card) = raven_familiar_to_return {
                    debug!(
                        "[Turn {turn:002}][Action]: Bouncing \"Raven Familiar\" back to hand.",
                        turn = self.turn
                    );
                    card.borrow_mut().zone = Zone::Hand;
                    return;
                }

                // Otherwise we must bounce the Harpy back to hand
                debug!(
                    "[Turn {turn:002}][Action]: Bouncing \"Cavern Harpy\" back to hand.",
                    turn = self.turn
                );
                game_object.borrow_mut().zone = Zone::Hand;
            }
            Effect::DamageEach(damage) => {
                self.damage_each(damage as i32);
            }
            Effect::Intuition => {
                let mut found = strategy.select_intuition(self);
                let found_str = found
                    .iter()
                    .map(|card| format!("\"{}\"", card.borrow().name))
                    .collect::<Vec<_>>()
                    .join(", ");

                debug!("[Turn {turn:002}][Action]: Searched for cards: {found_str} with Intuition.",
                    turn = self.turn);
                
                if let Some(card) = found.pop() {
                    self.deck.remove(&card);
                    card.borrow_mut().zone = Zone::Hand;

                    debug!("[Turn {turn:002}][Action]: Put \"{card_name}\" to hand.",
                        card_name = card.borrow().name,
                        turn = self.turn);
                }

                for card in found.into_iter() {
                    self.deck.remove(&card);
                    card.borrow_mut().zone = Zone::Graveyard;

                    debug!("[Turn {turn:002}][Action]: Put \"{card_name}\" to graveyard.",
                        card_name = card.borrow().name,
                        turn = self.turn);
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn mana_sources_count(&self) -> usize {
        self.game_objects
            .iter()
            .filter(|card| {
                let card = card.borrow();
                card.zone == Zone::Battlefield && !card.produced_mana.is_empty()
            })
            .count()
    }

    pub fn cleanup(&mut self, strategy: &Box<dyn Strategy>) {
        let cards_to_discard = strategy.discard_to_hand_size(self, 7);
        if !cards_to_discard.is_empty() {
            debug!(
                "[Turn {turn:002}][Action]: Discarding to hand size: {discard_str}",
                turn = self.turn,
                discard_str = cards_to_discard
                    .iter()
                    .map(|card| format!("\"{}\"", card.borrow().name))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }

        for card in cards_to_discard {
            card.borrow_mut().zone = Zone::Graveyard;
        }

        self.floating_mana.clear();
    }

    pub fn begin_turn(&mut self) {
        self.available_land_drops = 1;
        self.turn += 1;
    }

    pub fn find_starting_hand(&mut self, strategy: &Box<dyn Strategy>) {
        let mut mulligan_count = 0;

        loop {
            // Draw the starting hand
            self.draw_n(7);
            self.print_hand();
            if strategy.is_keepable_hand(self, mulligan_count) {
                debug!(
                    "[Turn {turn:002}][Action]: Keeping a hand of {cards} cards.",
                    turn = self.turn,
                    cards = 7 - mulligan_count
                );
                let bottomed = strategy.discard_to_hand_size(self, 7 - mulligan_count);

                if !bottomed.is_empty() {
                    let bottomed_str = bottomed
                        .iter()
                        .map(|card| card.borrow().name.clone())
                        .collect::<Vec<_>>()
                        .join(", ");
                    debug!("[Turn {turn:002}][Action]: Putting {count} cards on bottom: {bottomed_str}",
                        count = bottomed.len(),
                        turn = self.turn);
                }

                for card in bottomed {
                    // Return the cards to library
                    card.borrow_mut().zone = Zone::Library;
                    self.deck.put_bottom(card.clone())
                }
                break;
            } else {
                let hand = self
                    .game_objects
                    .iter()
                    .filter(is_hand)
                    .map(Rc::clone)
                    .collect::<Vec<_>>();

                for card in hand {
                    card.borrow_mut().zone = Zone::Library;
                    self.deck.put_bottom(card.clone());
                }

                self.deck.shuffle();
            }
            mulligan_count += 1;
            debug!(
                "[Turn {turn:002}][Action]: Taking a mulligan number {mulligan_count}.",
                turn = self.turn
            );
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.life_total -= amount;
        self.print_life();
    }

    pub fn deal_damage(&mut self, amount: i32) {
        self.damage_dealt += amount;
        self.print_life();
    }

    pub fn damage_each(&mut self, amount: i32) {
        self.life_total -= amount;
        self.damage_dealt += amount;
        self.print_life();
    }

    pub fn float_mana(&mut self) {
        // Produce colors in this priority order for now, producing 2 of each color first
        // TODO: Consider life loss here
        let colors = [Mana::Green, Mana::Blue, Mana::Black, Mana::White, Mana::Red];

        for land in self.game_objects.iter().filter(|card| {
            is_battlefield(card) && is_card_type(card, CardType::Land) && !is_tapped(card)
        }) {
            let mut land_used = false;
            // First try to produce colors we have the least of
            for color in colors.iter() {
                let land_name = land.borrow().name.clone();
                let floating = self.floating_mana.entry(*color).or_insert(0);

                if *floating < 2 {
                    if let Some(mana) = land.borrow().produced_mana.get(color) {
                        debug!(
                            "[Turn {turn:002}][Action]: Floating {mana} {color:?} mana from \"{land_name}\".",
                            turn = self.turn
                        );
                        *floating += mana;
                        land_used = true;
                        break;
                    }
                }
            }
            if !land_used {
                // Then fall back to just producing some mana the land produces
                for (color, mana) in land.borrow().produced_mana.iter() {
                    let land_name = land.borrow().name.clone();
                    let floating = self.floating_mana.entry(*color).or_insert(0);

                    debug!(
                        "[Turn {turn:002}][Action]: Floating {mana} {color:?} mana from \"{land_name}\".",
                        turn = self.turn
                    );
                    *floating += mana;
                    land_used = true;
                    break;
                }
            }

            land.borrow_mut().is_tapped = land_used;
        }
    }

    pub fn print_game_state(&self) {
        self.print_life();
        self.print_library();
        self.print_hand();
        self.print_battlefield();
        self.print_graveyard();
    }

    pub fn print_life(&self) {
        debug!(
            "[Turn {turn:002}][Game]: Life total: {life}, Damage dealt: {damage}",
            life = self.life_total,
            damage = self.damage_dealt,
            turn = self.turn,
        );
    }

    fn print_battlefield(&self) {
        let battlefield_str = self
            .game_objects
            .iter()
            .filter(is_battlefield)
            .map(|card| card.borrow().name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        debug!(
            "[Turn {turn:002}][Battlefield]: {battlefield_str}",
            turn = self.turn
        );
    }

    fn print_graveyard(&self) {
        let graveyard_str = self
            .game_objects
            .iter()
            .filter(is_graveyard)
            .map(|card| card.borrow().name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        debug!(
            "[Turn {turn:002}][Graveyard]: {graveyard_str}",
            turn = self.turn
        );

        let exile_str = self
            .game_objects
            .iter()
            .filter(is_exile)
            .map(|card| card.borrow().name.clone())
            .collect::<Vec<_>>()
            .join(", ");

        if !exile_str.is_empty() {
            debug!("[Turn {turn:002}][Exile]: {exile_str}", turn = self.turn);
        }
    }

    fn print_library(&self) {
        debug!(
            "[Turn {turn:002}][Library]: {deck} cards remaining.",
            turn = self.turn,
            deck = self.deck.len()
        );
    }

    fn print_hand(&self) {
        let hand_str = self
            .game_objects
            .iter()
            .filter(is_hand)
            .map(|card| card.borrow().name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        debug!("[Turn {turn:002}][Hand]: {hand_str}", turn = self.turn);
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;
    use crate::card::{Card};
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    #[test]
    fn it_avoids_using_limited_use_lands() {
        let mut game_objects = vec![
            Card::new_with_zone("Forest", Zone::Battlefield),
            Card::new_with_zone("Elvish Spirit Guide", Zone::Hand),
            Card::new_with_zone("Lotus Petal", Zone::Battlefield),
            Card::new_with_zone("Llanowar Wastes", Zone::Battlefield),
            Card::new_with_zone("Gemstone Mine", Zone::Battlefield),
            Card::new_with_zone("City of Brass", Zone::Battlefield),
            Card::new_with_zone("Llanowar Elves", Zone::Hand),
        ];

        // Should work in any order
        game_objects.shuffle(&mut thread_rng());

        let game = GameState {
            deck: Deck::from(Decklist { maindeck: vec![], sideboard: vec![] }),
            game_objects,
            turn: 0,
            life_total: 20,
            damage_dealt: 0,
            floating_mana: HashMap::new(),
            is_first_player: true,
            available_land_drops: 1,
        };

        let expected_order = [
            "Forest",
            "Elvish Spirit Guide",
            "Llanowar Wastes",
            "City of Brass",
            "Gemstone Mine",
            "Lotus Petal",
        ];

        for expected_source in expected_order {
            let castable = game.find_castable();
            let expected_cast = castable
                .iter()
                .find(|payment| payment.0.borrow().name == "Llanowar Elves");

            assert_eq!(true, expected_cast.is_some());

            let payment = &expected_cast.unwrap().1.as_ref().unwrap().0;

            assert_eq!(1, payment.len());
            assert_eq!(expected_source, payment[0].borrow().name);

            // Make this source spent and see what would be used next
            let mut spent = payment[0].borrow_mut();

            if spent.name == "Elvish Spirit Guide" {
                spent.zone = Zone::Exile;
            } else {
                spent.is_tapped = true;
            }
        }
    }

    #[test]
    fn it_plays_taplands_correctly() {
        let tapland = Card::new_with_zone("Hickory Woodlot", Zone::Hand);
        let llanowar_elves = Card::new_with_zone("Llanowar Elves", Zone::Hand);

        let mut game = GameState {
            deck: Deck::from(Decklist { maindeck: vec![], sideboard: vec![] }),
            game_objects: vec![tapland.clone(), llanowar_elves.clone()],
            turn: 0,
            life_total: 20,
            damage_dealt: 0,
            floating_mana: HashMap::new(),
            is_first_player: true,
            available_land_drops: 1,
        };

        game.play_land(tapland.clone());
        
        assert_eq!(Zone::Battlefield, tapland.borrow().zone);
        assert_eq!(true, tapland.borrow().is_tapped);

        let castable = game.find_castable();
        assert_eq!(true, castable.is_empty());
    }
}
