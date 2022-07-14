use std::cell::RefCell;
use std::collections::vec_deque::Iter;
use std::collections::VecDeque;
use std::rc::Rc;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::card::{Card, CardRef};

pub struct Decklist {
    pub maindeck: Vec<(&'static str, usize)>,
    pub sideboard: Vec<(&'static str, usize)>
}

#[derive(Clone, Debug, Default)]
pub struct Deck {
    maindeck: VecDeque<CardRef>,
    sideboard: Vec<CardRef>,
}

impl From<Decklist> for Deck {
    fn from(decklist: Decklist) -> Deck {
        let mut maindeck = Vec::with_capacity(60);
        let mut sideboard = Vec::with_capacity(15);

        for (card_name, quantity) in decklist.maindeck {
            let card = Card::new(card_name);

            for _ in 0..quantity {
                maindeck.push(Rc::new(RefCell::new(card.clone())));
            }
        }

        for (card_name, quantity) in decklist.sideboard {
            let card = Card::new(card_name);

            for _ in 0..quantity {
                sideboard.push(Rc::new(RefCell::new(card.clone())));
            }
        }

        Deck {
            maindeck: VecDeque::from(maindeck),
            sideboard
        }
    }
}

impl Deck {
    pub fn draw(&mut self) -> Option<CardRef> {
        self.maindeck.pop_back()
    }

    pub fn len(&self) -> usize {
        self.maindeck.len()
    }

    pub fn is_empty(&self) -> bool {
        self.maindeck.len() > 0
    }

    pub fn shuffle(&mut self) {
        let mut deck = Vec::from(self.maindeck.clone());
        deck.shuffle(&mut thread_rng());
        self.maindeck = VecDeque::from(deck);
    }

    pub fn search(&mut self, card_name: &str) -> Option<CardRef> {
        self.maindeck
            .iter()
            .position(|card| card.borrow().name == card_name)
            .and_then(|index| self.maindeck.remove(index))
    }

    pub fn search_sideboard(&mut self, card_name: &str) -> Option<CardRef> {
        self.sideboard
            .iter()
            .position(|card| card.borrow().name == card_name)
            .map(|index| self.sideboard.remove(index))
    }

    pub fn put_bottom(&mut self, card: CardRef) {
        self.maindeck.push_front(card)
    }

    pub fn put_top(&mut self, card: CardRef) {
        self.maindeck.push_back(card)
    }

    pub fn iter(&self) -> Iter<'_, CardRef> {
        self.maindeck.iter()
    }
}
