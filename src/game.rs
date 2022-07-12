use log::debug;
use std::collections::HashMap;
use std::rc::Rc;

use crate::card::{CardRef, CardType, Effect, Zone};
use crate::deck::Deck;
use crate::mana::find_payment_for;
use crate::mana::{Mana, PaymentAndFloating};
use crate::strategy::Strategy;
use crate::utils::*;

pub struct GameState {
    pub turn: usize,
    pub game_objects: Vec<CardRef>,
    pub available_land_drops: usize,
    deck: Deck,
    floating_mana: HashMap<Mana, usize>,
    is_first_player: bool,
}

impl GameState {
    pub fn new(decklist: Vec<(&str, usize)>) -> Self {
        let mut deck = Deck::new(decklist);

        let mut game_objects = Vec::with_capacity(deck.len());
        for card in deck.iter() {
            game_objects.push(card.clone())
        }

        deck.shuffle();

        Self {
            deck,
            game_objects,
            turn: 0,
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

        mana_sources.sort_by(sort_by_produced_mana);
        let castable = nonlands_in_hand
            .map(|card| {
                (
                    card.clone(),
                    find_payment_for(card.clone(), &mana_sources, self.floating_mana.clone()),
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
        card: &CardRef,
        (payment, floating): &PaymentAndFloating,
        attach_to: Option<CardRef>,
    ) {
        let mana_sources = payment
            .iter()
            .map(|source| source.borrow().name.clone())
            .collect::<Vec<_>>()
            .join(", ");

        let target_str = match attach_to.as_ref() {
            Some(target) => format!(" on target \"{}\"", target.borrow().name.clone()),
            None => "".to_owned(),
        };

        debug!("[Turn {turn:002}][Action]: Casting spell \"{card_name}\"{target_str} with mana sources: {mana_sources}",
            turn = self.turn,
            card_name = card.borrow().name);

        let new_zone = match card.borrow().card_type {
            CardType::Creature | CardType::Enchantment | CardType::Land | CardType::Artifact => {
                Zone::Battlefield
            }
            CardType::Sorcery | CardType::Instant => Zone::Graveyard,
        };

        card.borrow_mut().zone = new_zone;
        card.borrow_mut().attached_to = attach_to;

        if card.borrow().card_type == CardType::Creature {
            card.borrow_mut().is_summoning_sick = true;
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

        let effect = match card.borrow().on_resolve.clone() {
            Some(it) => it,
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
    use crate::card::{Card, CardRef};
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    #[test]
    fn it_avoids_using_limited_use_lands() {
        let llanowar_elves = Card::new_as_ref("Llanowar Elves");
        let gemstone_mine = Card::new_as_ref("Gemstone Mine");
        let city_of_brass = Card::new_as_ref("City of Brass");

        llanowar_elves.borrow_mut().zone = Zone::Hand;

        gemstone_mine.borrow_mut().zone = Zone::Battlefield;
        city_of_brass.borrow_mut().zone = Zone::Battlefield;

        let mut game_objects = vec![
            gemstone_mine.clone(),
            city_of_brass.clone(),
            gemstone_mine.clone(),
            city_of_brass.clone(),
            llanowar_elves,
        ];

        // Should work in any order
        game_objects.shuffle(&mut thread_rng());

        let game = GameState {
            deck: Deck::new(vec![]),
            game_objects,
            turn: 0,
            floating_mana: HashMap::new(),
            is_first_player: true,
            available_land_drops: 1,
        };

        let castable = game.find_castable();

        assert_eq!(1, castable.len());
        assert_eq!(true, castable[0].1.is_some());

        let payment: &Vec<CardRef> = &castable.first().as_ref().unwrap().1.as_ref().unwrap().0;
        assert_eq!(1, payment.len());
        assert_eq!("City of Brass", payment[0].borrow().name);
    }
}