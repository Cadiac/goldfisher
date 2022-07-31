use std::collections::HashMap;

use crate::card::{CardRef, CardType};
use crate::deck::Decklist;
use crate::game::{Game, GameResult, GameStatus};
use crate::utils::*;

pub mod aluren;
pub mod pattern_hulk;

pub trait Strategy {
    fn name(&self) -> String;
    fn default_decklist(&self) -> Decklist;
    fn game_status(&self, game: &Game) -> GameStatus {
        if game.life_total <= 0 && game.damage_dealt >= 20 {
            return GameStatus::Finished(GameResult::Draw);
        }

        if game.life_total <= 0 {
            return GameStatus::Finished(GameResult::Lose);
        }

        if game.damage_dealt >= 20 {
            return GameStatus::Finished(GameResult::Win);
        }

        GameStatus::Continue
    }

    fn is_keepable_hand(&self, game: &Game, mulligan_count: usize) -> bool;
    fn take_game_action(&self, game: &mut Game) -> bool;
    fn play_land(&self, game: &mut Game) -> bool {
        if game.available_land_drops > 0 {
            let mut lands_in_hand = game
                .game_objects
                .iter()
                .filter(|card| is_hand(card) && is_card_type(card, CardType::Land))
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
        game.game_objects
            .iter()
            .filter(is_library)
            .take(3)
            .cloned()
            .collect()
    }

    fn discard_to_hand_size(&self, game: &Game, hand_size: usize) -> Vec<CardRef>;
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::card::{Card, Zone};
    use crate::deck::{Deck, Decklist};
    use crate::strategy::pattern_hulk::{PatternHulk};
    
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
            game_objects,
            turn: 0,
            life_total: 20,
            damage_dealt: 0,
            floating_mana: HashMap::new(),
            is_first_player: true,
            available_land_drops: 10,
        };

        let strategy = PatternHulk{};

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
