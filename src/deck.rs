use std::cell::RefCell;
use std::collections::vec_deque::Iter;
use std::collections::VecDeque;
use std::rc::Rc;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::card::{Card, CardRef};

#[derive(Clone, Debug, Default)]
pub struct Deck(VecDeque<CardRef>);

impl From<Vec<CardRef>> for Deck {
    fn from(cards: Vec<CardRef>) -> Deck {
        Deck(VecDeque::from(cards))
    }
}

impl Deck {
    pub fn new(decklist: Vec<(&str, usize)>) -> Self {
        let mut deck = Vec::with_capacity(60);

        for (card_name, quantity) in decklist {
            let card = Card::new(card_name);

            for _ in 0..quantity {
                deck.push(Rc::new(RefCell::new(card.clone())));
            }
        }

        deck.into()
    }

    pub fn draw(&mut self) -> Option<CardRef> {
        self.0.pop_back()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() > 0
    }

    pub fn shuffle(&mut self) {
        let mut deck = Vec::from(self.0.clone());
        deck.shuffle(&mut thread_rng());
        self.0 = VecDeque::from(deck);
    }

    pub fn search(&mut self, card_name: &str) -> Option<CardRef> {
        self.0
            .iter()
            .position(|card| card.borrow().name == card_name)
            .and_then(|index| self.0.remove(index))
    }

    pub fn put_bottom(&mut self, card: CardRef) {
        self.0.push_front(card)
    }

    pub fn put_top(&mut self, card: CardRef) {
        self.0.push_back(card)
    }

    pub fn iter(&self) -> Iter<'_, CardRef> {
        self.0.iter()
    }
}
