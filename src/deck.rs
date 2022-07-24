use std::cell::RefCell;
use std::collections::vec_deque::Iter;
use std::collections::VecDeque;
use std::error::Error;
use std::rc::Rc;
use std::str::FromStr;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::card::{Card, CardRef, Zone};

pub struct Decklist {
    pub maindeck: Vec<(String, usize)>,
    pub sideboard: Vec<(String, usize)>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ParseDeckError(String);

impl Error for ParseDeckError {}

impl std::fmt::Display for ParseDeckError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "failed to parse deck: {}", self.0)
    }
}

impl FromStr for Decklist {
    type Err = ParseDeckError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut maindeck = Vec::with_capacity(60);
        let mut sideboard = Vec::with_capacity(15);

        let mut is_maindeck = true;

        for (index, line) in s.lines().enumerate() {
            if line.starts_with("//") {
                continue;
            }

            if line.is_empty() {
                is_maindeck = false;
                continue;
            }

            let (quantity_str, card_name) = line.split_once(" ").ok_or_else(|| {
                ParseDeckError(format!(
                    "on line {line_number}: malformed quantity and name: \"{line}\"",
                    line_number = index + 1
                ))
            })?;

            let quantity = quantity_str.parse::<usize>().or_else(|msg| {
                Err(ParseDeckError(format!(
                    "on line {line_number}: failed to parse quantity: {msg}",
                    line_number = index + 1
                )))
            })?;

            if is_maindeck {
                maindeck.push((card_name.to_owned(), quantity));
            } else {
                sideboard.push((card_name.to_owned(), quantity));
            }
        }

        Ok(Decklist {
            maindeck,
            sideboard,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Deck {
    pub maindeck: VecDeque<CardRef>,
    pub sideboard: Vec<CardRef>,
}

impl Deck {
    pub fn new(decklist: Decklist) -> Result<Self, ParseDeckError> {
        let mut maindeck = Vec::with_capacity(60);
        let mut sideboard = Vec::with_capacity(15);

        for (card_name, quantity) in decklist.maindeck {
            let card = Card::new(&card_name).or_else(|msg| {
                Err(ParseDeckError(format!("failed to create deck: {msg}")))
            })?;

            for _ in 0..quantity {
                maindeck.push(Rc::new(RefCell::new(card.clone())));
            }
        }

        for (card_name, quantity) in decklist.sideboard {
            let mut card = Card::new(&card_name).or_else(|msg| {
                Err(ParseDeckError(format!("failed to create deck: {msg}")))
            })?;
            card.zone = Zone::Outside;

            for _ in 0..quantity {
                sideboard.push(Rc::new(RefCell::new(card.clone())));
            }
        }

        Ok(Deck {
            maindeck: VecDeque::from(maindeck),
            sideboard,
        })
    }
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

    pub fn remove(&mut self, card: &CardRef) -> Option<CardRef> {
        self.maindeck
            .iter()
            .position(|deck_card| Rc::ptr_eq(deck_card, card))
            .and_then(|index| self.maindeck.remove(index))
    }

    pub fn remove_sideboard(&mut self, card: &CardRef) -> Option<CardRef> {
        self.sideboard
            .iter()
            .position(|side_card| Rc::ptr_eq(side_card, card))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_decklist() {
        let decklist = "4 Llanowar Elves\n\
            1 Birds of Paradise\n\
            2 Forest\n\
            1 Swamp\n\
            5 Island\n\
            \n\
            // Sideboard\n\
            2 Engineered Plague\n\
            3 Naturalize\n";

        let result = decklist.parse::<Decklist>();
        assert_eq!(true, result.is_ok());
        let deck = result.unwrap();

        assert_eq!(13, deck.maindeck.len());
        assert_eq!(5, deck.sideboard.len());
        assert_eq!(
            vec![
                (String::from("Llanowar Elves"), 4),
                (String::from("Birds of Paradise"), 1),
                (String::from("Forest"), 2),
                (String::from("Swamp"), 1),
                (String::from("Island"), 5),
            ],
            deck.maindeck
        );
        assert_eq!(
            vec![
                (String::from("Engineered Plague"), 2),
                (String::from("Naturalize"), 3),
            ],
            deck.sideboard
        );
    }

    #[test]
    fn it_handles_malformed_lines() {
        let decklist = "1 Birds of Paradise\n\
            BrokenLine\n\
            4 Llanowar Elves";

        let result = decklist.parse::<Decklist>();
        assert_eq!(
            Some(ParseDeckError(
                "on line 2: malformed quantity and name: \"BrokenLine\"".to_owned()
            )),
            result.err()
        );
    }

    #[test]
    fn it_handles_malformed_quantity() {
        let decklist = "1 Birds of Paradise\n\
            4foobar Llanowar Elves\n\
            20 Forest";

        let result = decklist.parse::<Decklist>();
        assert_eq!(
            Some(ParseDeckError(
                "on line 2: failed to parse quantity: invalid digit found in string".to_owned()
            )),
            result.err()
        );
    }
}
