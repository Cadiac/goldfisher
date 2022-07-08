use std::cell::RefCell;
use std::rc::Rc;

use crate::card::{Card, CardType, Zone};
use crate::deck::Deck;
use crate::mana::find_payment_for;

pub struct GameState {
    pub deck: Deck,
    pub turn: usize,
    pub game_objects: Vec<Rc<RefCell<Card>>>,
    pub is_first_player: bool,
}

impl GameState {
    pub fn new(deck: Deck) -> Self {
        Self {
            deck,
            turn: 0,
            game_objects: Vec::with_capacity(60),
            is_first_player: true,
        }
    }

    pub fn is_win_condition_met(&self) -> bool {
        let creatures = self
            .game_objects
            .iter()
            .filter(|card| {
                let card = card.borrow();
                card.zone == Zone::Battlefield && card.card_type == CardType::Creature
            })
            .collect::<Vec<_>>();
        
        let sac_outlets = self
            .game_objects
            .iter()
            .filter(|card| {
                let card = card.borrow();
                card.zone == Zone::Battlefield && card.is_sac_outlet
            })
            .collect::<Vec<_>>();

        let rectors = self
            .game_objects
            .iter()
            .filter(|card| {
                let card = card.borrow();
                card.zone == Zone::Battlefield && card.is_rector
            })
            .collect::<Vec<_>>();

        let is_pattern_attached_to_redundant_creature = self.game_objects.iter().any(|card| {
            let card = card.borrow();

            if card.zone != Zone::Battlefield || !card.is_pattern {
                false
            } else {
                match &card.attached_to {
                    Some(target) => {
                        if target.borrow().is_sac_outlet {
                            // Make sure we have a redundant sac outlet if pattern is attached to one
                            sac_outlets.len() > 1
                        } else {
                            true
                        }
                    }
                    None => false,
                }
            }
        });

        // Winning combinations:
        // 1) Sac outlet + any redundant creature + Pattern of Rebirth on that creature
        if !sac_outlets.is_empty() && is_pattern_attached_to_redundant_creature {
            return true;
        }

        // 2) Sac outlet + Academy Rector + any redundant creature
        if !sac_outlets.is_empty() && !rectors.is_empty() && creatures.len() >= 3 {
            return true;
        }

        // TODO: Check for more wincons involving Cabal Therapy

        return false

    }

    pub fn cast_creatures(&self) {
        let castable = self.find_castable();

        let creature = castable
            .iter()
            .find(|(card, _)| card.borrow().card_type == CardType::Creature);
        if let Some((card_ref, payment)) = creature {
            self.cast_spell(card_ref, payment.as_ref().unwrap(), None);
        }
    }

    pub fn cast_sac_outlets(&self) {
        let castable = self.find_castable();

        let sac_creature = castable.iter().find(|(card, _)| {
            let card = card.borrow();
            card.card_type == CardType::Creature && card.is_sac_outlet
        });
        if let Some((card_ref, payment)) = sac_creature {
            self.cast_spell(card_ref, payment.as_ref().unwrap(), None);
        }
    }

    pub fn cast_pattern_of_rebirths(&self) {
        let castable = self.find_castable();

        let pattern_of_rebirth = castable.iter().find(|(card, _)| card.borrow().is_pattern);
        let is_creature_on_battlefield = self.game_objects.iter().any(|card| {
            let card = card.borrow();
            card.zone == Zone::Battlefield && card.card_type == CardType::Creature
        });
        let is_pattern_on_battlefield = self.game_objects.iter().any(|card| {
            let card = card.borrow();
            card.zone == Zone::Battlefield && card.is_pattern
        });
        if let Some((card_ref, payment)) = pattern_of_rebirth {
            if payment.is_some() && is_creature_on_battlefield && !is_pattern_on_battlefield {
                // Target non-sacrifice outlets over sac outlets
                let non_sac_creature = self.game_objects.iter().find(|card| {
                    let card = card.borrow();
                    card.zone == Zone::Battlefield
                        && card.card_type == CardType::Creature
                        && !card.is_sac_outlet
                });

                let target = if let Some(creature) = non_sac_creature {
                    Rc::clone(creature)
                } else {
                    // Otherwise just cast it on a sac outlet
                    let sac_creature = self.game_objects.iter().find(|card| {
                        let card = card.borrow();
                        card.zone == Zone::Battlefield
                            && card.card_type == CardType::Creature
                            && card.is_sac_outlet
                    });

                    Rc::clone(sac_creature.unwrap())
                };

                self.cast_spell(card_ref, payment.as_ref().unwrap(), Some(target));
            }
        }
    }

    pub fn cast_rectors(&self) {
        let castable = self.find_castable();

        let rector = castable.iter().find(|(card, _)| card.borrow().is_rector);
        let is_pattern_on_battlefield = self.game_objects.iter().any(|card| {
            let card = card.borrow();
            card.zone == Zone::Battlefield && card.is_pattern
        });

        if let Some((card_ref, payment)) = rector {
            if payment.is_some() && !is_pattern_on_battlefield {
                self.cast_spell(card_ref, payment.as_ref().unwrap(), None);
            }
        }
    }

    pub fn find_castable(
        &self,
    ) -> Vec<(Rc<RefCell<Card>>, Option<(Vec<Rc<RefCell<Card>>>, usize)>)> {
        let nonlands_in_hand = self.game_objects.iter().filter(|card| {
            let card = card.borrow();
            card.zone == Zone::Hand && card.card_type != CardType::Land
        });
        let mut mana_sources: Vec<_> = self
            .game_objects
            .iter()
            .filter(|card| {
                let card = card.borrow();
                card.zone == Zone::Battlefield
                    && !card.produced_mana.is_empty()
                    && !card.is_summoning_sick
                    && !card.is_tapped
            })
            .map(Rc::clone)
            .collect();
        mana_sources.sort_by(|a, b| {
            a.borrow()
                .produced_mana
                .len()
                .partial_cmp(&b.borrow().produced_mana.len())
                .unwrap()
        });
        let castable = nonlands_in_hand
            .map(|card| {
                (
                    card.clone(),
                    find_payment_for(&card.borrow(), &mana_sources),
                )
            })
            .filter(|(_, payment)| payment.is_some());

        castable.collect()
    }

    pub fn play_land(&self) {
        let mut lands_in_hand = self
            .game_objects
            .iter()
            .filter(|card| {
                let card = card.borrow();
                card.zone == Zone::Hand && card.card_type == CardType::Land
            })
            .map(|card| card)
            .collect::<Vec<_>>();
        lands_in_hand.sort_by(|a, b| {
            a.borrow()
                .produced_mana
                .len()
                .partial_cmp(&b.borrow().produced_mana.len())
                .unwrap()
        });
        // Play the one that produces most colors
        // TODO: Play the one that produces most cards that could be played
        if let Some(land) = lands_in_hand.pop() {
            let mut card = land.borrow_mut();

            println!(
                "[Turn {turn:002}][Action]: Playing land: \"{land}\"",
                turn = self.turn,
                land = card.name
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
            if let Some(mut card) = self.deck.draw() {
                card.zone = Zone::Hand;
                self.game_objects.push(Rc::new(RefCell::new(card)))
            }
        }
    }

    pub fn untap(&self) {
        for card in self.game_objects.iter() {
            let mut card = card.borrow_mut();
            card.is_summoning_sick = false;
            card.is_tapped = false;
        }
    }

    pub fn print_game_state(&self) {
        let hand_str = self
            .game_objects
            .iter()
            .filter(|card| card.borrow().zone == Zone::Hand)
            .map(|card| card.borrow().name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let battlefield_str = self
            .game_objects
            .iter()
            .filter(|card| card.borrow().zone == Zone::Battlefield)
            .map(|card| card.borrow().name.clone())
            .collect::<Vec<_>>()
            .join(", ");

        println!(
            "[Turn {turn:002}][Library]: {deck} cards remaining.",
            turn = self.turn,
            deck = self.deck.len()
        );
        println!("[Turn {turn:002}][Hand]: {hand_str}", turn = self.turn);
        println!(
            "[Turn {turn:002}][Battlefield]: {battlefield_str}",
            turn = self.turn
        );
    }

    pub fn cast_spell(
        &self,
        card_ref: &Rc<RefCell<Card>>,
        (payment, _floating): &(Vec<Rc<RefCell<Card>>>, usize),
        attach_to: Option<Rc<RefCell<Card>>>,
    ) {
        let mut card = card_ref.borrow_mut();

        let card_name = &card.name;
        let mana_sources = payment
            .iter()
            .map(|source| source.borrow().name.clone())
            .collect::<Vec<_>>()
            .join(", ");

        println!("[Turn {turn:002}][Action]: Casting spell \"{card_name}\" on target: {target:?} with: {mana_sources}",
            turn = self.turn,
            target = attach_to.as_ref().map(|card| card.borrow().name.clone()));

        card.attached_to = attach_to;
        card.zone = Zone::Battlefield;

        if card.card_type == CardType::Creature {
            card.is_summoning_sick = true;
        }

        for mana_source in payment {
            mana_source.borrow_mut().is_tapped = true;
        }
    }

    pub fn cleanup(&self) {
        let cards_in_hand = self
            .game_objects
            .iter()
            .filter(|card| card.borrow().zone == Zone::Hand)
            .enumerate();

        for (index, card) in cards_in_hand {
            if index >= 7 {
                // TODO: Select the cards to discard by priority
                card.borrow_mut().zone = Zone::Graveyard;
            }
        }
    }

    pub fn advance_turn(&mut self) {
        self.turn += 1;
    }
}
