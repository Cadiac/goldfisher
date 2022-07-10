use log::debug;
use std::collections::HashMap;
use std::rc::Rc;

use crate::card::{CardRef, CardType, Zone};
use crate::deck::{Deck};
use crate::mana::find_payment_for;
use crate::mana::{Mana, PaymentAndFloating};

pub struct GameState {
    pub turn: usize,
    deck: Deck,
    game_objects: Vec<CardRef>,
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
        }
    }

    pub fn is_win_condition_met(&self) -> bool {
        let creatures = self
            .game_objects
            .iter()
            .filter(|card| is_battlefield(card) && is_creature(card))
            .count();

        let rectors = self
            .game_objects
            .iter()
            .filter(|card| is_battlefield(card) && is_rector(card))
            .count();

        let sac_outlets = self
            .game_objects
            .iter()
            .filter(|card| is_battlefield(card) && is_sac_outlet(card))
            .count();

        let cabal_therapies_in_graveyard = self
            .game_objects
            .iter()
            .filter(|card| is_graveyard(card) && is_named(card, "Cabal Therapy"))
            .count();

        let patterns = self
            .game_objects
            .iter()
            .filter(|card| is_battlefield(card) && is_pattern(card))
            .count();

        let is_pattern_attached_to_redundant_creature = self.game_objects.iter().any(|card| {
            if is_battlefield(&card) && is_pattern(&card) {
                let card = card.borrow();

                match &card.attached_to {
                    Some(target) => {
                        if target.borrow().is_sac_outlet {
                            // Make sure we have a redundant sac outlet if pattern is attached to one
                            sac_outlets > 1
                        } else {
                            true
                        }
                    }
                    None => false,
                }
            } else {
                false
            }
        });

        let untapped_phyrexian_tower = self.game_objects.iter().any(|card| {
            is_battlefield(&card) && is_named(&card, "Phyrexian Tower") && !is_tapped(&card)
        });

        // Winning combinations:
        // 1) Sac outlet + any redundant creature + Pattern of Rebirth on that creature
        if sac_outlets >= 1 && is_pattern_attached_to_redundant_creature {
            return true;
        }

        // 2) Sac outlet + Academy Rector + any redundant creature
        if sac_outlets >= 1 && rectors >= 1 && creatures >= 3 {
            return true;
        }

        // 3) At least one Academy Rector + Pattern of Rebirth on a creature + Cabal Therapy in graveyard / Phyrexian Tower
        // TODO: Make sure we still have Goblin Bombardment in library
        if rectors >= 1 && patterns >= 1 && (cabal_therapies_in_graveyard >= 1 || untapped_phyrexian_tower) {
            return true;
        }

        // 4) At least two Academy Rectors + Cabal Therapy in graveyard / Phyrexian Tower + at least three creatures total
        if rectors >= 2 && creatures >= 3 && (cabal_therapies_in_graveyard >= 1 || untapped_phyrexian_tower) {
            return true;
        }

        false
    }

    pub fn cast_mana_dorks(&mut self) {
        let castable = self.find_castable();

        let mut mana_dorks = castable
            .iter()
            .filter(|(card, _)| is_mana_dork(&card))
            .collect::<Vec<_>>();

        // Cast the one that produces most colors
        mana_dorks.sort_by(|(a, _), (b, _)| sort_by_produced_mana(a, b));

        if let Some((card_ref, payment)) = mana_dorks.last() {
            self.cast_spell(card_ref, payment.as_ref().unwrap(), None);
        }
    }

    pub fn cast_sac_outlets(&mut self) {
        let castable = self.find_castable();

        let mut sac_outlets = castable
            .iter()
            .filter(|(card, _)| is_sac_outlet(&card))
            .collect::<Vec<_>>();

        // Cast the cheapest first
        sac_outlets.sort_by(|(a, _), (b, _)| sort_by_cmc(a, b));

        if let Some((card_ref, payment)) = sac_outlets.first() {
            self.cast_spell(card_ref, payment.as_ref().unwrap(), None);
        }
    }

    pub fn cast_pattern_of_rebirths(&mut self) {
        let castable = self.find_castable();

        let is_creature_on_battlefield = self
            .game_objects
            .iter()
            .any(|card| is_battlefield(&card) && is_creature(&card));
        let is_pattern_on_battlefield = self
            .game_objects
            .iter()
            .any(|card| is_battlefield(&card) && is_pattern(&card));

        let pattern_of_rebirth = castable.iter().find(|(card, _)| card.borrow().is_pattern);

        if let Some((card_ref, payment)) = pattern_of_rebirth {
            if payment.is_some() && is_creature_on_battlefield && !is_pattern_on_battlefield {
                // Target non-sacrifice outlets over sac outlets
                let non_sac_creature = self
                    .game_objects
                    .iter()
                    .find(|card| is_battlefield(card) && is_creature(card) && !is_sac_outlet(card));

                let target = if let Some(creature) = non_sac_creature {
                    Rc::clone(creature)
                } else {
                    // Otherwise just cast it on a sac outlet
                    let sac_creature = self.game_objects.iter().find(|card| {
                        is_battlefield(card) && is_creature(card) && is_sac_outlet(card)
                    });

                    Rc::clone(sac_creature.unwrap())
                };

                self.cast_spell(card_ref, payment.as_ref().unwrap(), Some(target));
            }
        }
    }

    pub fn cast_rectors(&mut self) {
        let castable = self.find_castable();

        let rector = castable.iter().find(|(card, _)| card.borrow().is_rector);
        let is_pattern_on_battlefield = self
            .game_objects
            .iter()
            .any(|card| is_battlefield(&card) && is_pattern(&card));

        if let Some((card_ref, payment)) = rector {
            if payment.is_some() && !is_pattern_on_battlefield {
                self.cast_spell(card_ref, payment.as_ref().unwrap(), None);
            }
        }
    }

    pub fn cast_redundant_creatures(&mut self) {
        let castable = self.find_castable();

        let mut creatures = castable
            .iter()
            .filter(|(c, _)| is_creature(&c))
            .collect::<Vec<_>>();

        // Cast the cheapest creatures first
        creatures.sort_by(|(a, _), (b, _)| sort_by_cmc(a, b));

        if let Some((card_ref, payment)) = creatures.first() {
            self.cast_spell(card_ref, payment.as_ref().unwrap(), None);
        }
    }

    pub fn cast_others(&mut self) {
        loop {
            let mut castable = self.find_castable();

            if castable.is_empty() {
                return;
            }

            // Cast the cheapest first
            castable.sort_by(|(a, _), (b, _)| sort_by_cmc(a, b));

            if let Some((card_ref, payment)) = castable.first() {
                self.cast_spell(card_ref, payment.as_ref().unwrap(), None);
            }
        }
    }

    fn find_castable(&self) -> Vec<(CardRef, Option<PaymentAndFloating>)> {
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

        mana_sources.sort_by(sort_by_produced_mana);
        let castable = nonlands_in_hand
            .map(|card| {
                (
                    card.clone(),
                    find_payment_for(&card.borrow(), &mana_sources, self.floating_mana.clone()),
                )
            })
            .filter(|(_, payment)| payment.is_some());

        castable.collect()
    }

    pub fn play_land(&self) {
        let mut lands_in_hand = self
            .game_objects
            .iter()
            .filter(|card| is_hand(card) && is_land(card))
            .collect::<Vec<_>>();

        lands_in_hand.sort_by(|a, b| sort_by_produced_mana(a, b));

        // Play the one that produces most colors
        // TODO: Play the one that produces most cards that could be played
        if let Some(land) = lands_in_hand.last() {
            let mut card = land.borrow_mut();

            debug!(
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
            if let Some(card) = self.deck.draw() {
                card.borrow_mut().zone = Zone::Hand;
            } else {
                panic!("empty library!");
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
        self.print_graveyard();
    }

    pub fn cast_spell(
        &mut self,
        card_ref: &CardRef,
        (payment, floating): &PaymentAndFloating,
        attach_to: Option<CardRef>,
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
            None => "".to_owned(),
        };

        debug!("[Turn {turn:002}][Action]: Casting spell \"{card_name}\"{target_str} with mana sources: {mana_sources}",
            turn = self.turn);

        card.zone = match card.card_type {
            CardType::Creature | CardType::Enchantment | CardType::Land | CardType::Artifact => {
                Zone::Battlefield
            }
            CardType::Sorcery => Zone::Graveyard,
        };

        card.attached_to = attach_to;

        if card.card_type == CardType::Creature {
            card.is_summoning_sick = true;
        }

        for mana_source in payment {
            mana_source.borrow_mut().is_tapped = true;
        }

        self.floating_mana = floating.to_owned();
    }

    pub fn cleanup(&mut self) {
        let cards_to_discard = self.select_worst_cards(7);

        for card in cards_to_discard {
            card.borrow_mut().zone = Zone::Graveyard;
        }

        self.floating_mana.clear();
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
                debug!(
                    "[Turn {turn:002}][Action]: Keeping a hand of {cards} cards.",
                    turn = self.turn,
                    cards = 7 - mulligan_count
                );
                let bottomed = self.select_worst_cards(7 - mulligan_count);

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

    fn is_keepable_hand(&self, mulligan_count: usize) -> bool {
        if mulligan_count >= 3 {
            // Just keep the hand with 4 cards
            return true;
        }

        let hand = self.game_objects.iter().filter(is_hand);

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

        if (is_pattern_in_hand || is_rector_in_hand || is_sac_outlet_in_hand) && creatures_count > 0
        {
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

    fn select_worst_cards(&self, hand_size: usize) -> Vec<CardRef> {
        let mut ordered_hand = Vec::new();
        let mut lands = Vec::with_capacity(7);
        let mut patterns_or_rectors = Vec::with_capacity(7);
        let mut mana_dorks = Vec::with_capacity(7);
        let mut sac_outlets = Vec::with_capacity(7);
        let mut redundant_cards = Vec::with_capacity(7);

        let hand = self.game_objects.iter().filter(is_hand);

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
        // Prefer lands that produce the most colors of mana (sorted to the end of the iter)
        let mut lands_iter = lands.iter().rev();
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

        ordered_hand
            .iter()
            .skip(hand_size)
            .map(|card| Rc::clone(card))
            .collect()
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
        let battlefield_str = self
            .game_objects
            .iter()
            .filter(is_graveyard)
            .map(|card| card.borrow().name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        debug!(
            "[Turn {turn:002}][Graveyard]: {battlefield_str}",
            turn = self.turn
        );
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

// Utility functions

fn is_battlefield(card: &&CardRef) -> bool {
    card.borrow().zone == Zone::Battlefield
}

fn is_hand(card: &&CardRef) -> bool {
    card.borrow().zone == Zone::Hand
}

fn is_graveyard(card: &&CardRef) -> bool {
    card.borrow().zone == Zone::Graveyard
}

fn is_creature(card: &&CardRef) -> bool {
    card.borrow().card_type == CardType::Creature
}

fn is_land(card: &&CardRef) -> bool {
    card.borrow().card_type == CardType::Land
}

fn is_rector(card: &&CardRef) -> bool {
    card.borrow().is_rector
}

fn is_pattern(card: &&CardRef) -> bool {
    card.borrow().is_pattern
}

fn is_sac_outlet(card: &&CardRef) -> bool {
    card.borrow().is_sac_outlet
}

fn is_mana_dork(card: &&CardRef) -> bool {
    let card = card.borrow();
    card.card_type == CardType::Creature && !card.produced_mana.is_empty()
}

fn is_named(card: &&CardRef, name: &str) -> bool {
    card.borrow().name == name
}

fn is_tapped(card: &&CardRef) -> bool {
    card.borrow().is_tapped
}

fn sort_by_produced_mana(a: &CardRef, b: &CardRef) -> std::cmp::Ordering {
    a.borrow()
        .produced_mana
        .len()
        .partial_cmp(&b.borrow().produced_mana.len())
        .unwrap()
}

fn sort_by_cmc(a: &CardRef, b: &CardRef) -> std::cmp::Ordering {
    a.borrow()
        .cost
        .values()
        .sum::<usize>()
        .partial_cmp(&b.borrow().cost.values().sum())
        .unwrap()
}
