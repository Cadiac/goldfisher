use std::collections::HashMap;

use crate::card::{CardRef, CardType};
use crate::deck::Decklist;
use crate::game::{GameState, GameStatus};
use crate::utils::*;

pub mod aluren;
pub mod pattern_rector;

pub trait Strategy {
    fn game_status(&self, game: &GameState) -> GameStatus;
    fn is_keepable_hand(&self, game: &GameState, mulligan_count: usize) -> bool;
    fn take_game_action(&self, game: &mut GameState) -> bool;
    fn play_land(&self, game: &mut GameState) -> bool {
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
    fn find_best_card(
        &self,
        game: &GameState,
        cards: HashMap<String, Vec<CardRef>>,
    ) -> Option<CardRef>;
    fn discard_to_hand_size(&self, game: &GameState, hand_size: usize) -> Vec<CardRef>;
    fn decklist() -> Decklist;
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::card::{Card, Zone};
    use crate::deck::{Deck, Decklist};
    use crate::strategy::pattern_rector::{PatternRector};
    
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

        let mut game = GameState {
            deck: Deck::from(Decklist { maindeck: vec![], sideboard: vec![] }),
            game_objects,
            turn: 0,
            life_total: 20,
            floating_mana: HashMap::new(),
            is_first_player: true,
            available_land_drops: 10,
        };

        let strategy = PatternRector{};

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
