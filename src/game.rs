use std::cell::RefCell;
use std::rc::Rc;

use crate::card::{Card, CardType, Zone};
use crate::deck::Deck;
use crate::mana::{find_payment_for};

pub struct GameState {
    pub deck: Deck,
    pub turn: usize,
    pub game_objects: Vec<Rc<RefCell<Card>>>,
    pub is_first_player: bool,
}

impl GameState {
    pub fn new(mut deck: Deck) -> Self {
        deck.shuffle();

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

        // TODO: Check for more wincons involving Cabal Therapy and Phyrexian Tower

        return false

    }

    pub fn cast_mana_dorks(&self) {
        let castable = self.find_castable();

        let mut mana_dorks = castable
            .iter()
            .filter(|(card, _)| {
                let card = card.borrow();
                card.card_type == CardType::Creature && !card.produced_mana.is_empty()
            })
            .collect::<Vec<_>>();

        // Pick the one that produces most colors
        mana_dorks.sort_by(|(a, _), (b, _)| sort_by_produced_mana(a,b));

        if let Some((card_ref, payment)) = mana_dorks.first() {
            self.cast_spell(card_ref, payment.as_ref().unwrap(), None);
        }
    }

    pub fn cast_redundant_creatures(&self) {
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

        let sac_outlet = castable.iter().find(|(card, _)| card.borrow().is_sac_outlet);
        if let Some((card_ref, payment)) = sac_outlet {
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

    fn find_castable(
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
        self.print_library();
        self.print_hand();
        self.print_battlefield();
    }

    pub fn print_battlefield(&self) {
        let battlefield_str = self
            .game_objects
            .iter()
            .filter(|card| card.borrow().zone == Zone::Battlefield)
            .map(|card| card.borrow().name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "[Turn {turn:002}][Battlefield]: {battlefield_str}",
            turn = self.turn
        );
    }

    pub fn print_library(&self) {
        println!(
            "[Turn {turn:002}][Library]: {deck} cards remaining.",
            turn = self.turn,
            deck = self.deck.len()
        );
    }    

    pub fn print_hand(&self) {
        let hand_str = self
            .game_objects
            .iter()
            .filter(|card| card.borrow().zone == Zone::Hand)
            .map(|card| card.borrow().name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        println!("[Turn {turn:002}][Hand]: {hand_str}", turn = self.turn);
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

        let target_str = match attach_to.as_ref() {
            Some(target) => format!(" on target \"{}\"", target.borrow().name.clone()),
            None => "".to_owned()
        };

        println!("[Turn {turn:002}][Action]: Casting spell \"{card_name}\"{target_str} with mana sources: {mana_sources}",
            turn = self.turn);

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

    pub fn mana_sources_count(&self) -> usize {
        self.game_objects
            .iter()
            .filter(|card| {
                let card = card.borrow();
                card.zone == Zone::Battlefield && !card.produced_mana.is_empty()
            })
            .count()
    }

    pub fn find_starting_hand(&mut self) {
        let mut mulligan_count = 0;

        loop {
            // Draw the starting hand
            self.draw_n(7);
            self.print_hand();
            if self.is_keepable_hand(mulligan_count) {
                println!("[Turn {turn:002}][Action]: Keeping a hand of {cards} cards.", turn = self.turn, cards = 7 - mulligan_count);
                let bottomed = self.select_worst_cards(7 - mulligan_count);
                
                if bottomed.len() > 0 {
                    let bottomed_str = bottomed.iter()
                        .map(|card| card.borrow().name.clone())
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("[Turn {turn:002}][Action]: Putting {count} cards on bottom: {bottomed_str}",
                        count = bottomed.len(),
                        turn = self.turn);
                }

                for card in bottomed {
                    // Remove the cards from game objects
                    self.game_objects.retain(|game_object| !Rc::ptr_eq(&card, game_object));
                    self.deck.put_bottom(card.borrow().clone())
                }
                break;
            } else {
                let hand = self.game_objects.iter()
                    .filter(|card| card.borrow().zone == Zone::Hand)
                    .map(Rc::clone)
                    .collect::<Vec<_>>();

                for card in hand {
                    self.deck.put_bottom(card.borrow().clone());
                    self.game_objects.retain(|game_object| !Rc::ptr_eq(game_object, &card));
                }

                self.deck.shuffle();
            }
            mulligan_count += 1;
            println!("[Turn {turn:002}][Action]: Taking a mulligan number {mulligan_count}.", turn = self.turn);
        }
    }

    fn is_keepable_hand(&self, mulligan_count: usize) -> bool {
        if mulligan_count >= 3 {
            // Just keep the hand with 4 cards
            return true
        }

        let hand = self.game_objects
            .iter()
            .filter(|card| card.borrow().zone == Zone::Hand);

        let mut is_pattern_in_hand = false;
        let mut is_rector_in_hand = false;
        let mut is_sac_outlet_in_hand = false;

        let mut creatures_count = 0;
        let mut lands_count = 0;
        let mut mana_dorks_count = 0;

        for card in hand {
            let card = card.borrow();

            if card.is_pattern {
                is_pattern_in_hand = true;
            }

            if card.is_rector {
                is_rector_in_hand = true;
            }

            if card.is_sac_outlet {
                is_sac_outlet_in_hand = true;
            }

            if card.card_type == CardType::Creature {
                creatures_count += 1;
            }

            if card.card_type == CardType::Land {
                lands_count += 1;
            }

            if card.card_type == CardType::Creature && !card.produced_mana.is_empty() {
                mana_dorks_count += 1;
            }
        }

        if lands_count == 0 {
            // Always mulligan zero land hands
            return false;
        }

        if lands_count >= 6 {
            // Also mulligan too land heavy hands
            return false;
        }

        if lands_count == 1 && mana_dorks_count <= 1 {
            // One landers with just max one mana dork get automatically mulliganed too
            return false;
        }

        // Having a rector/pattern and sac outlet in hand is always good
        if (is_pattern_in_hand || is_rector_in_hand) && is_sac_outlet_in_hand {
            return true;
        }

        
        if (is_pattern_in_hand || is_rector_in_hand || is_sac_outlet_in_hand) && creatures_count > 0 {
            // If we have already taken any mulligans this should be good enough
            if mulligan_count > 0 {
                return true;
            }
            
            // At full hand one of the combo pieces with reasonable mana is a keep
            if lands_count + mana_dorks_count >= 3 {
                return true;
            }
        }

        // Otherwise take a mulligan
        false
    }

    fn select_worst_cards(&self, hand_size: usize) -> Vec<Rc<RefCell<Card>>> {
        let mut ordered_hand = Vec::new();
        let mut lands = Vec::with_capacity(7);
        let mut patterns_or_rectors = Vec::with_capacity(7);
        let mut mana_dorks = Vec::with_capacity(7);
        let mut sac_outlets = Vec::with_capacity(7);
        let mut redundant_cards = Vec::with_capacity(7);

        let hand = self.game_objects
            .iter()
            .filter(|card| card.borrow().zone == Zone::Hand);

        for card in hand {
            let c = card.borrow();

            if c.card_type == CardType::Land {
                lands.push(card.clone());
            } else if c.is_pattern || c.is_rector {
                patterns_or_rectors.push(card.clone());
            } else if c.is_sac_outlet {
                sac_outlets.push(card.clone());
            } else if c.card_type == CardType::Creature && !c.produced_mana.is_empty() {
                mana_dorks.push(card.clone());
            } else {
                redundant_cards.push(card.clone());
            }
        }

        lands.sort_by(sort_by_produced_mana);
        sac_outlets.sort_by(sort_by_cmc);

        // First take a balanced mix of lands and combo pieces
        let mut lands_iter = lands.iter();
        for _ in 0..2 {
            if let Some(card) = lands_iter.next() {
                ordered_hand.push(card);
            }
        }

        let mut patterns_or_rectors_iter = patterns_or_rectors.iter();
        for _ in 0..1 {
            if let Some(card) = patterns_or_rectors_iter.next() {
                ordered_hand.push(card);
            }
        }

        let mut sac_outlets_iter = sac_outlets.iter();
        for _ in 0..1 {
            if let Some(card) = sac_outlets_iter.next() {
                ordered_hand.push(card);
            }
        }

        // Take all mana dorks over extra lands for quick kills
        for card in mana_dorks.iter() {
            ordered_hand.push(card);
        }

        // Then take the rest of the cards, still in priority order
        for card in lands_iter {
            ordered_hand.push(card);
        }
        for card in patterns_or_rectors_iter {
            ordered_hand.push(card);
        }
        for card in sac_outlets_iter {
            ordered_hand.push(card);
        }
        for card in redundant_cards.iter() {
            ordered_hand.push(card);
        }
        
        ordered_hand.iter()
            .skip(hand_size)
            .map(|card| Rc::clone(card))
            .collect()
    }
}

fn sort_by_produced_mana(a: &Rc<RefCell<Card>>, b: &Rc<RefCell<Card>>) -> std::cmp::Ordering {
    b.borrow()
        .produced_mana
        .len()
        .partial_cmp(&a.borrow().produced_mana.len())
        .unwrap()
}

fn sort_by_cmc(a: &Rc<RefCell<Card>>, b: &Rc<RefCell<Card>>) -> std::cmp::Ordering {
    a.borrow()
        .cost.values().sum::<usize>()
        .partial_cmp(&b.borrow().cost.values().sum())
        .unwrap()
}
