use log::debug;
use std::collections::HashMap;
use std::rc::Rc;

use crate::card::{CardRef, CardType, Effect, SearchFilter, Zone};
use crate::deck::{Deck, Decklist};
use crate::mana::find_payment_for;
use crate::mana::{Mana, PaymentAndFloating};
use crate::strategy::Strategy;
use crate::utils::*;

pub struct GameState {
    pub turn: usize,
    pub game_objects: Vec<CardRef>,
    pub available_land_drops: usize,
    pub deck: Deck,
    pub life_total: i32,
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

    pub fn draw(&mut self) {
        if self.turn == 0 || (self.turn == 1 && !self.is_first_player) || self.turn > 1 {
            if let Some(card) = self.deck.draw() {
                let mut card = card.borrow_mut();

                card.zone = Zone::Hand;
                debug!(
                    "[Turn {turn:002}][Action]: Drew card \"{land}\".",
                    turn = self.turn,
                    land = card.name
                );
            } else {
                panic!("empty library!");
            }
        }
    }

    pub fn untap(&mut self) {
        for card in self.game_objects.iter() {
            let mut card = card.borrow_mut();
            card.is_summoning_sick = false;
            card.is_tapped = false;
        }
    }

    pub fn take_game_actions(&mut self, strategy: &impl Strategy) -> bool {
        loop {
            let action_taken = strategy.take_game_action(self);
            if strategy.is_win_condition_met(self) {
                return true;
            }
            if !action_taken {
                return false;
            }
        }
    }

    pub fn cast_spell(
        &mut self,
        strategy: &impl Strategy,
        source: &CardRef,
        (payment, floating): &PaymentAndFloating,
        attach_to: Option<CardRef>,
    ) {
        let mana_sources = payment
            .iter()
            .map(|mana_source| mana_source.borrow().name.clone())
            .collect::<Vec<_>>()
            .join(", ");

        let target_str = match attach_to.as_ref() {
            Some(target) => format!(" on target \"{}\"", target.borrow().name.clone()),
            None => "".to_owned(),
        };

        debug!("[Turn {turn:002}][Action]: Casting card \"{card_name}\"{target_str} with mana sources: {mana_sources}",
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

        let effect = match source.borrow().on_resolve.clone() {
            Some(e) => e,
            _ => return,
        };

        match effect {
            Effect::SearchAndPutTopOfLibrary(card_filter) => {
                let card_name = strategy.best_card_to_draw(self, card_filter).to_owned();
                match self.deck.search(&card_name) {
                    Some(found) => {
                        debug!("[Turn {turn:002}][Action]: Searched for \"{card_name}\" and put it on top of the library.",
                            turn = self.turn,
                            card_name = found.borrow().name);
                        self.deck.shuffle();
                        self.deck.put_top(found)
                    }
                    None => debug!(
                        "[Turn {turn:002}][Action]: Failed to find.",
                        turn = self.turn
                    ),
                }
            }
            Effect::SearchAndPutHand(card_filter) => {
                if card_filter == Some(SearchFilter::LivingWish) {
                    let card_name = strategy.best_card_to_draw(self, card_filter).to_owned();

                    match self.deck.search_sideboard(&card_name) {
                        Some(found) => {
                            debug!("[Turn {turn:002}][Action]: Searched for \"{card_name}\" from sideboard and put it in hand.",
                                turn = self.turn,
                                card_name = found.borrow().name);

                            found.borrow_mut().zone = Zone::Hand;
                            self.game_objects.push(found);
                        }
                        None => debug!(
                            "[Turn {turn:002}][Action]: Failed to find.",
                            turn = self.turn
                        ),
                    }
                } else {
                    let card_name = strategy.best_card_to_draw(self, card_filter).to_owned();

                    match self.deck.search(&card_name) {
                        Some(found) => {
                            debug!("[Turn {turn:002}][Action]: Searched for \"{card_name}\" and put it in hand.",
                                turn = self.turn,
                                card_name = found.borrow().name);
                            self.deck.shuffle();

                            found.borrow_mut().zone = Zone::Hand;
                        }
                        None => debug!(
                            "[Turn {turn:002}][Action]: Failed to find.",
                            turn = self.turn
                        ),
                    }
                }
            }
            Effect::Impulse => {
                // TODO: Proper impulse, selecting the best draw
                self.draw()
            }
            Effect::UntapLands(n) => {
                for _ in 0..n {
                    let mut tapped_lands = self
                        .game_objects
                        .iter()
                        .filter(|card| is_battlefield(card) && is_land(card) && is_tapped(card))
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
                let etb_draw_triggers = self
                    .game_objects
                    .iter()
                    .filter(|card| {
                        let card = card.borrow();
                        card.zone == Zone::Battlefield && card.name == "Wirewood Savage"
                    })
                    .count();

                if etb_draw_triggers > 0 {
                    for _ in 0..etb_draw_triggers {
                        if self.deck.len() > 0 {
                            self.draw();
                        }
                    }

                    source.borrow_mut().zone = Zone::Hand;
                    return
                }

                let maggot_carrier_to_return = self
                    .game_objects
                    .iter()
                    .find(|card| {
                        let card = card.borrow();
                        card.zone == Zone::Battlefield && card.name == "Maggot Carrier"
                    });

                if let Some(card) = maggot_carrier_to_return {
                    debug!(
                        "[Turn {turn:002}][Action]: Bouncing \"Maggot Carrier\" back to hand.",
                        turn = self.turn
                    );
                    card.borrow_mut().zone = Zone::Hand;
                }


                // TODO: Decide whether we want to untap or untap lands?

                let raven_familiar_to_return = self
                    .game_objects
                    .iter()
                    .find(|card| {
                        let card = card.borrow();
                        card.zone == Zone::Battlefield && card.name == "Raven Familiar"
                    });

                if let Some(card) = raven_familiar_to_return {
                    debug!(
                        "[Turn {turn:002}][Action]: Bouncing \"Raven Familiar\" back to hand.",
                        turn = self.turn
                    );
                    card.borrow_mut().zone = Zone::Hand;
                }

                // let cloud_of_faeries_to_return = self
                //     .game_objects
                //     .iter()
                //     .find(|card| {
                //         let card = card.borrow();
                //         card.zone == Zone::Battlefield && card.name == "Cloud of Faeries"
                //     });

                // if let Some(card) = cloud_of_faeries_to_return {
                //     debug!(
                //         "[Turn {turn:002}][Action]: Bouncing \"Cloud of Faeries\" back to hand.",
                //         turn = self.turn
                //     );
                //     card.borrow_mut().zone = Zone::Hand;
                //     should_bounce_self = true;
                // }

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

    pub fn cleanup(&mut self, strategy: &impl Strategy) {
        let cards_to_discard = strategy.worst_cards_in_hand(self, 7);

        for card in cards_to_discard {
            card.borrow_mut().zone = Zone::Graveyard;
        }

        self.floating_mana.clear();
    }

    pub fn begin_turn(&mut self) {
        self.available_land_drops = 1;
        self.turn += 1;
    }

    pub fn find_starting_hand(&mut self, strategy: &impl Strategy) {
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
                let bottomed = strategy.worst_cards_in_hand(self, 7 - mulligan_count);

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

    pub fn print_game_state(&self) {
        self.print_library();
        self.print_hand();
        self.print_battlefield();
        self.print_graveyard();
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
}
