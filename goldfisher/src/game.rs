use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;

use crate::card::{CardRef, CardType, SubType, Zone};
use crate::deck::{Deck, Decklist, ParseDeckError};
use crate::mana::find_payment_for;
use crate::mana::{Mana, PaymentAndFloating};
use crate::strategy::Strategy;
use crate::utils::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameResult {
    Win,
    Lose,
    Draw,
}

pub enum GameStatus {
    Continue,
    Finished(GameResult),
}

#[derive(Default)]
pub struct Game {
    pub turn: usize,
    pub game_objects: Vec<CardRef>,
    pub available_land_drops: usize,
    pub deck: Deck,
    pub life_total: i32,
    pub damage_dealt: i32,
    pub opponent_library: i32,
    pub floating_mana: HashMap<Mana, u32>,
    pub is_first_player: bool,
    pub mulligan_count: usize,
    pub storm: usize,
}

impl Game {
    /// Creates a new game with given decklist
    pub fn new(decklist: &Decklist) -> Result<Self, ParseDeckError> {
        let mut deck = Deck::new(decklist)?;

        let mut game_objects = Vec::with_capacity(deck.len());
        for card in deck.iter() {
            game_objects.push(card.clone())
        }

        debug!("Deck size: {deck_size}", deck_size = deck.len());

        deck.shuffle();

        Ok(Self {
            deck,
            game_objects,
            turn: 0,
            life_total: 20,
            damage_dealt: 0,
            opponent_library: 60,
            floating_mana: HashMap::new(),
            is_first_player: true,
            available_land_drops: 1,
            mulligan_count: 0,
            storm: 0,
        })
    }

    /// Runs the game to completion.
    ///
    /// ```
    /// use goldfisher::strategy::{pattern_hulk, Strategy};
    /// use goldfisher::game::{Game};
    ///
    /// let mut strategy: Box<dyn Strategy> = Box::new(pattern_hulk::PatternHulk {});
    /// let mut game = Game::new(&strategy.default_decklist()).unwrap();
    ///
    /// game.run(&mut strategy);
    /// ```
    pub fn run(&mut self, strategy: &mut Box<dyn Strategy>) -> (GameResult, usize, usize) {
        debug!("====================[ START OF GAME ]=======================");

        self.find_starting_hand(strategy);

        let result = loop {
            self.begin_turn();

            debug!(
                "======================[ TURN {turn:002} ]===========================",
                turn = self.turn
            );

            self.untap();

            if let GameStatus::Finished(result) = self.draw() {
                break result;
            }

            self.print_game_state();

            if let GameStatus::Finished(result) = self.take_game_actions(strategy) {
                break result;
            }

            if let GameStatus::Finished(result) = self.cleanup(strategy) {
                break result;
            }
        };

        debug!("=====================[ END OF GAME ]========================");
        debug!(
            "                    {result:?} on turn {turn}!",
            turn = self.turn
        );
        debug!("============================================================");
        self.print_game_state();

        (result, self.turn, self.mulligan_count)
    }

    /// Finds all castable game objects with their payments and floating mana left over afterwards.
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

        let cost_reductions = self
            .game_objects
            .iter()
            .filter(|card| is_battlefield(card) && is_cost_reducer(card))
            .map(|card| card.borrow().cost_reduction.as_ref().unwrap().clone())
            .collect::<Vec<_>>();

        let castable = nonlands_in_hand
            .map(|card| {
                (
                    card.clone(),
                    find_payment_for(
                        card.clone(),
                        &mana_sources,
                        self.floating_mana.clone(),
                        &cost_reductions,
                    ),
                )
            })
            .filter(|(_, payment)| payment.is_some());

        castable.collect()
    }

    /// Plays a land drop if possible.
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

    /// Draw `amount` cards from the library.
    /// If there aren't enough cards to draw the game ends in a loss.
    pub fn draw_n(&mut self, amount: usize) -> GameStatus {
        for _ in 0..amount {
            let status = self.draw();
            if let GameStatus::Finished(_) = status {
                return status;
            }
        }

        GameStatus::Continue
    }

    /// Draw a card from the library.
    /// If there aren't enough cards to draw the game ends in a loss.
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
                return GameStatus::Finished(GameResult::Lose);
            }
        }
        GameStatus::Continue
    }

    /// Untaps all the lands and clears summoning sickness
    pub fn untap(&mut self) {
        for card in self.game_objects.iter() {
            let mut card = card.borrow_mut();

            if card.zone == Zone::Battlefield {
                card.is_summoning_sick = false;
                card.is_tapped = false;
            }
        }
    }

    /// Takes game actions until no further actions are taken or game ends
    pub fn take_game_actions(&mut self, strategy: &mut Box<dyn Strategy>) -> GameStatus {
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

    /// Casts the spell, paying its cost with the payment.
    /// The payment has to be fresh, as this function trusts that it is a valid payment
    /// at the time the spell is cast.
    pub fn cast_spell(
        &mut self,
        strategy: &impl Strategy,
        source: &CardRef,
        (payment, floating): &PaymentAndFloating,
        attach_to: Option<CardRef>,
    ) {
        self.storm += 1;

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
            let has_haste = source.borrow().is_haste;
            source.borrow_mut().is_summoning_sick = !has_haste;

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

    /// Applies any effects to the game the game object resolving might cause
    pub fn handle_on_resolve_effects(&mut self, source: &CardRef, strategy: &impl Strategy) {
        let on_resolve = source.borrow().on_resolve.clone();
        if let Some(effect) = on_resolve {
            effect.resolve(self, source, strategy)
        }
    }

    /// Returns the count of available mana sources on battlefield
    pub fn mana_sources_count(&self) -> usize {
        self.game_objects
            .iter()
            .filter(|card| {
                let card = card.borrow();
                card.zone == Zone::Battlefield && !card.produced_mana.is_empty()
            })
            .count()
    }

    pub fn discard(&mut self, card: CardRef) {
        debug!(
            "[Turn {turn:002}][Action]: Discarding card {card_name}",
            turn = self.turn,
            card_name = card.borrow().name,
        );
        card.borrow_mut().zone = Zone::Graveyard;
    }

    /// Cleanup phase, discards cards to hand size
    pub fn cleanup(&mut self, strategy: &mut Box<dyn Strategy>) -> GameStatus {
        let cards_to_discard = strategy.discard_to_hand_size(self, 7);
        if !cards_to_discard.is_empty() {
            debug!(
                "[Turn {turn:002}][Action]: Discarding to hand size",
                turn = self.turn,
            );
        }

        for card in cards_to_discard {
            self.discard(card);
        }

        self.floating_mana.clear();
        strategy.cleanup();

        // Opponent is taking a turn an drawing from potentially empty library.
        // Count this as a win for this turn.
        self.opponent_library -= 1;
        if self.opponent_library < 0 {
            debug!(
                "[Turn {turn:002}][Game]: Opponent began their turn and drew from empty library",
                turn = self.turn
            );
            return GameStatus::Finished(GameResult::Win);
        }

        GameStatus::Continue
    }

    /// Begins the turn, resetting land drops and advancing turn counter
    pub fn begin_turn(&mut self) {
        self.available_land_drops = 1;
        self.storm = 0;
        self.turn += 1;
    }

    /// Takes starting hands and decides whether to keep or mulligan them based on the strategy.
    pub fn find_starting_hand(&mut self, strategy: &Box<dyn Strategy>) {
        // Assume opponent also draws 7 and keeps
        self.opponent_library -= 7;

        loop {
            // Draw the starting hand
            self.draw_n(7);
            self.print_hand();
            if strategy.is_keepable_hand(self, self.mulligan_count) {
                debug!(
                    "[Turn {turn:002}][Action]: Keeping a hand of {cards} cards.",
                    turn = self.turn,
                    cards = 7 - self.mulligan_count
                );
                let bottomed = strategy.discard_to_hand_size(self, 7 - self.mulligan_count);

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
            self.mulligan_count += 1;
            debug!(
                "[Turn {turn:002}][Action]: Taking a mulligan number {mulligan_count}.",
                mulligan_count = self.mulligan_count,
                turn = self.turn
            );
        }
    }

    /// Deals `amount` damage to self
    pub fn take_damage(&mut self, amount: i32) {
        self.life_total -= amount;
        self.print_life();
    }

    /// Deals `amount` damage to the opponent
    pub fn deal_damage(&mut self, amount: i32) {
        self.damage_dealt += amount;
        self.print_life();
    }

    /// Deals `amount` damage to both players
    pub fn damage_each(&mut self, amount: i32) {
        self.life_total -= amount;
        self.damage_dealt += amount;
        self.print_life();
    }

    /// Floats mana from all lands, trying to produce even amount of colors
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
            "[Turn {turn:002}][Game]: Life total: {life}, Damage dealt: {damage}, Opponent's library: {library}",
            life = self.life_total,
            damage = self.damage_dealt,
            library = self.opponent_library,
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

        let game = Game {
            game_objects,
            life_total: 20,
            is_first_player: true,
            available_land_drops: 1,
            ..Default::default()
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

        let mut game = Game {
            game_objects: vec![tapland.clone(), llanowar_elves.clone()],
            life_total: 20,
            is_first_player: true,
            available_land_drops: 1,
            ..Default::default()
        };

        game.play_land(tapland.clone());
        
        assert_eq!(Zone::Battlefield, tapland.borrow().zone);
        assert_eq!(true, tapland.borrow().is_tapped);

        let castable = game.find_castable();
        assert_eq!(true, castable.is_empty());
    }
}
