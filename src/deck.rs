use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::card::{Card, CardType};
use crate::mana::{Mana};

#[derive(Clone, Debug, Default)]
pub struct Deck(VecDeque<Card>);

impl From<Vec<Card>> for Deck {
    fn from(cards: Vec<Card>) -> Deck {
        Deck(VecDeque::from(cards))
    }
}

impl Deck {
    pub fn draw(&mut self) -> Option<Card> {
        self.0.pop_back()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn shuffle(&mut self) {
        let mut deck = Vec::from(self.0.clone());
        deck.shuffle(&mut thread_rng());
        self.0 = VecDeque::from(deck);
    }

    pub fn put_bottom(&mut self, card: Card) {
        self.0.push_front(card)
    }
}

pub fn create_deck(cards: Vec<(&str, usize)>) -> Deck {
    let mut deck = Vec::with_capacity(60);

    for (name, quantity) in cards {
        let card = match name {
            "Llanowar Elves" => Card {
                name: "Llanowar Elves".to_owned(),
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Green, 1)]),
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                ..Default::default()
            },
            "Birds of Paradise" => Card {
                name: "Birds of Paradise".to_owned(),
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Green, 1)]),
                produced_mana: HashMap::from([
                    (Mana::White, 1),
                    (Mana::Blue, 1),
                    (Mana::Black, 1),
                    (Mana::Red, 1),
                    (Mana::Green, 1),
                ]),
                ..Default::default()
            },
            "Carrion Feeder" => Card {
                name: "Carrion Feeder".to_owned(),
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Black, 1)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Nantuko Husk" => Card {
                name: "Nantuko Husk".to_owned(),
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Phyrexian Ghoul" => Card {
                name: "Phyrexian Ghoul".to_owned(),
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Pattern of Rebirth" => Card {
                name: "Pattern of Rebirth".to_owned(),
                card_type: CardType::Enchantment,
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 3)]),
                is_pattern: true,
                ..Default::default()
            },
            "Academy Rector" => Card {
                name: "Academy Rector".to_owned(),
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 3)]),
                is_rector: true,
                ..Default::default()
            },
            "Forest" => Card {
                name: "Forest".to_owned(),
                card_type: CardType::Land,
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                ..Default::default()
            },
            "Swamp" => Card {
                name: "Swamp".to_owned(),
                card_type: CardType::Land,
                produced_mana: HashMap::from([(Mana::Black, 1)]),
                ..Default::default()
            },
            "Plains" => Card {
                name: "Plains".to_owned(),
                card_type: CardType::Land,
                produced_mana: HashMap::from([(Mana::White, 1)]),
                ..Default::default()
            },
            _ => unimplemented!(),
        };

        for _ in 0..quantity {
            deck.push(card.clone());
        }
    }

    deck.into()
}
