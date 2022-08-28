use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::str::FromStr;

use crate::card::{CardRef, CardType};
use crate::deck::Decklist;
use crate::game::{Game, Outcome, GameStatus};
use crate::mana::{PaymentAndFloating};
use crate::utils::*;

pub mod aluren;
pub mod frantic_storm;
pub mod pattern_combo;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeckStrategy {
    PatternCombo,
    Aluren,
    FranticStorm,
}

impl FromStr for DeckStrategy {
    type Err = ();

    fn from_str(input: &str) -> Result<DeckStrategy, Self::Err> {
        match input {
            pattern_combo::NAME => Ok(DeckStrategy::PatternCombo),
            aluren::NAME => Ok(DeckStrategy::Aluren),
            frantic_storm::NAME => Ok(DeckStrategy::FranticStorm),
            _ => Err(()),
        }
    }
}

impl fmt::Display for DeckStrategy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DeckStrategy::PatternCombo => pattern_combo::NAME,
                DeckStrategy::Aluren => aluren::NAME,
                DeckStrategy::FranticStorm => frantic_storm::NAME,
            }
        )
    }
}

pub const STRATEGIES: &[DeckStrategy] = &[
    DeckStrategy::PatternCombo,
    DeckStrategy::Aluren,
    DeckStrategy::FranticStorm,
];

pub fn from_enum(strategy: &DeckStrategy) -> Box<dyn Strategy> {
    match strategy {
        DeckStrategy::PatternCombo => Box::new(pattern_combo::PatternCombo::new()),
        DeckStrategy::Aluren => Box::new(aluren::Aluren::new()),
        DeckStrategy::FranticStorm => Box::new(frantic_storm::FranticStorm::new()),
    }
}

pub trait Strategy {
    fn name(&self) -> String;
    fn default_decklist(&self) -> Decklist;
    fn cleanup(&mut self) {}

    fn game_status(&self, game: &Game) -> GameStatus {
        if game.life_total <= 0 && game.damage_dealt >= 20 {
            return GameStatus::Finished(Outcome::Draw);
        }

        if game.life_total <= 0 {
            return GameStatus::Finished(Outcome::Lose);
        }

        if game.damage_dealt >= 20 {
            return GameStatus::Finished(Outcome::Win);
        }

        if game.opponent_library <= 0 {
            return GameStatus::Finished(Outcome::Win);
        }

        GameStatus::Continue
    }

    fn is_keepable_hand(&self, game: &Game, mulligan_count: usize) -> bool;
    fn take_game_action(&mut self, game: &mut Game) -> bool;

    fn cast_named(
        &self,
        game: &mut Game,
        castable: Vec<(CardRef, PaymentAndFloating)>,
        card_name: &str,
    ) -> bool
    where
        Self: Sized,
    {
        if let Some((card_ref, payment)) =
            castable.iter().find(|(c, _)| c.borrow().name == card_name)
        {
            game.cast_spell(self, card_ref, payment, None);
            return true;
        }

        false
    }

    fn cast_mana_producers(&self, game: &mut Game) -> bool
    where
        Self: Sized,
    {
        let castable = game.find_castable();

        let mut mana_producers = castable
            .iter()
            .filter(|(card, _)| is_mana_dork(&card))
            .collect::<Vec<_>>();

        // Cast the one that produces most colors
        mana_producers.sort_by(|(a, _), (b, _)| sort_by_best_mana_to_play(a, b));

        if let Some((card_ref, payment)) = mana_producers.last() {
            game.cast_spell(self, card_ref, payment, None);
            return true;
        }

        false
    }

    fn play_land(&self, game: &mut Game) -> bool {
        if game.available_land_drops > 0 {
            let mut lands_in_hand = game
                .game_objects
                .iter()
                .filter(|card| is_hand(card) && is_card_type(card, &CardType::Land))
                .cloned()
                .collect::<Vec<_>>();

            lands_in_hand.sort_by(sort_by_best_mana_to_play);

            // Play the one that produces most colors
            // TODO: Play the one that produces most cards that could be played
            let best_land_in_hand = lands_in_hand.last().map(|card| (*card).clone());

            if let Some(land) = best_land_in_hand {
                game.play_land(land);
                return true;
            }
        }
        false
    }
    fn select_best(&self, game: &Game, cards: HashMap<String, Vec<CardRef>>) -> Option<CardRef>;

    fn select_intuition(&self, game: &Game) -> Vec<CardRef> {
        let searchable = apply_search_filter(game, &None);
        let mut selected = Vec::with_capacity(3);

        while let Some(found) = self.select_best(game, group_by_name(searchable.clone())) {
            selected.push(found);
        }

        selected
    }

    fn discard_to_hand_size(&self, game: &Game, hand_size: usize) -> Vec<CardRef> {
        let mut cards_to_discard: Vec<_> =
            game.game_objects.iter().filter(is_hand).cloned().collect();

        if cards_to_discard.len() <= hand_size {
            return Vec::new();
        }

        for _ in 0..hand_size {
            if let Some(best) = self.select_best(game, group_by_name(cards_to_discard.clone())) {
                cards_to_discard.retain(|card| !Rc::ptr_eq(card, &best));
            }
        }

        cards_to_discard
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;

    use crate::card::{Card, Zone};
    use crate::deck::{Deck, Decklist};
    use crate::strategy::pattern_combo::{PatternCombo};
    
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    #[test]
    fn it_plays_lands_with_unlimited_uses_first() {
        let mut game_objects = vec![
            Card::new_with_zone("City of Brass", Zone::Hand),
            Card::new_with_zone("Gemstone Mine", Zone::Hand),
            Card::new_with_zone("City of Brass", Zone::Hand),
            Card::new_with_zone("Gemstone Mine", Zone::Hand),
            Card::new_with_zone("City of Brass", Zone::Hand),
            Card::new_with_zone("Gemstone Mine", Zone::Hand),
            Card::new_with_zone("City of Brass", Zone::Hand),
            Card::new_with_zone("Gemstone Mine", Zone::Hand),
            Card::new_with_zone("City of Brass", Zone::Hand),
            Card::new_with_zone("Llanowar Wastes", Zone::Hand),
        ];

        // Should work in any order
        game_objects.shuffle(&mut thread_rng());

        let mut game = Game {
            deck: Deck::new(&Decklist { maindeck: vec![], sideboard: vec![] }).unwrap(),
            life_total: 20,
            is_first_player: true,
            available_land_drops: 10,
            game_objects,
            ..Default::default()
        };

        let strategy = PatternCombo{};

        for land_drops in 1..=10 {
            assert_eq!(true, strategy.play_land(&mut game));

            let on_battlefield = game.game_objects
                .iter()
                .filter(|card| card.borrow().zone == Zone::Battlefield)
                .collect::<Vec<_>>();

            assert_eq!(land_drops, on_battlefield.len());

            if land_drops <= 5 {
                assert_eq!(land_drops, on_battlefield.iter().filter(|card| card.borrow().name == "City of Brass").count());
            } else if land_drops <= 9 {
                assert_eq!(5, on_battlefield.iter().filter(|card| card.borrow().name == "City of Brass").count());
                assert_eq!(land_drops - 5, on_battlefield.iter().filter(|card| card.borrow().name == "Gemstone Mine").count());
            } else {
                assert_eq!(5, on_battlefield.iter().filter(|card| card.borrow().name == "City of Brass").count());
                assert_eq!(4, on_battlefield.iter().filter(|card| card.borrow().name == "Gemstone Mine").count());
                assert_eq!(1, on_battlefield.iter().filter(|card| card.borrow().name == "Llanowar Wastes").count());
            }
        }
    }
}
