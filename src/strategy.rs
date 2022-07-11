use crate::game::{GameState};
use crate::card::{CardRef, SearchFilter};

pub mod pattern_rector;

pub trait Strategy {
    fn is_win_condition_met(&self, game: &GameState) -> bool;
    fn is_keepable_hand(&self, game: &GameState, mulligan_count: usize) -> bool;
    fn take_game_action(&self, game: &mut GameState) -> bool;
    fn best_card_to_draw(&self, game: &GameState, search_filter: Option<SearchFilter>) -> &str;
    fn worst_cards_in_hand(&self, game: &GameState, hand_size: usize) -> Vec<CardRef>;
}
