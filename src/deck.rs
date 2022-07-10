use std::cell::RefCell;
use std::collections::vec_deque::Iter;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::card::{Card, CardRef, CardType};
use crate::mana::Mana;

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
            let card = match card_name {
                "Llanowar Elves" => Card {
                    name: "Llanowar Elves".to_owned(),
                    card_type: CardType::Creature,
                    cost: HashMap::from([(Mana::Green, 1)]),
                    produced_mana: HashMap::from([(Mana::Green, 1)]),
                    ..Default::default()
                },
                "Fyndhorn Elves" => Card {
                    name: "Fyndhorn Elves".to_owned(),
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
                "Mesmeric Fiend" => Card {
                    name: "Mesmeric Fiend".to_owned(),
                    card_type: CardType::Creature,
                    cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 1)]),
                    ..Default::default()
                },
                "Iridescent Drake" => Card {
                    name: "Iridescent Drake".to_owned(),
                    card_type: CardType::Creature,
                    cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 3)]),
                    ..Default::default()
                },
                "Karmic Guide" => Card {
                    name: "Karmic Guide".to_owned(),
                    card_type: CardType::Creature,
                    cost: HashMap::from([(Mana::White, 2), (Mana::Colorless, 3)]),
                    ..Default::default()
                },
                "Volrath's Shapeshifter" => Card {
                    name: "Volrath's Shapeshifter".to_owned(),
                    card_type: CardType::Creature,
                    cost: HashMap::from([(Mana::Blue, 2), (Mana::Colorless, 1)]),
                    ..Default::default()
                },
                "Caller of the Claw" => Card {
                    name: "Caller of the Claw".to_owned(),
                    card_type: CardType::Creature,
                    cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 2)]),
                    ..Default::default()
                },
                "Body Snatcher" => Card {
                    name: "Body Snatcher".to_owned(),
                    card_type: CardType::Creature,
                    cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 3)]),
                    ..Default::default()
                },
                "Akroma, Angel of Wrath" => Card {
                    name: "Akroma, Angel of Wrath".to_owned(),
                    card_type: CardType::Creature,
                    cost: HashMap::from([(Mana::White, 3), (Mana::Colorless, 5)]),
                    ..Default::default()
                },
                "Worship" => Card {
                    name: "Worship".to_owned(),
                    card_type: CardType::Enchantment,
                    cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 3)]),
                    ..Default::default()
                },
                "Goblin Bombardment" => Card {
                    name: "Goblin Bombardment".to_owned(),
                    card_type: CardType::Enchantment,
                    cost: HashMap::from([(Mana::Red, 1), (Mana::Colorless, 1)]),
                    is_sac_outlet: true,
                    ..Default::default()
                },
                "Altar of Dementia" => Card {
                    name: "Altar of Dementia".to_owned(),
                    card_type: CardType::Artifact,
                    cost: HashMap::from([(Mana::Colorless, 2)]),
                    is_sac_outlet: true,
                    ..Default::default()
                },
                "Cabal Therapy" => Card {
                    name: "Cabal Therapy".to_owned(),
                    card_type: CardType::Sorcery,
                    cost: HashMap::from([(Mana::Black, 1)]),
                    ..Default::default()
                },
                "Duress" => Card {
                    name: "Duress".to_owned(),
                    card_type: CardType::Sorcery,
                    cost: HashMap::from([(Mana::Black, 1)]),
                    ..Default::default()
                },
                "City of Brass" => Card {
                    name: "City of Brass".to_owned(),
                    card_type: CardType::Land,
                    produced_mana: HashMap::from([
                        (Mana::White, 1),
                        (Mana::Blue, 1),
                        (Mana::Black, 1),
                        (Mana::Red, 1),
                        (Mana::Green, 1),
                    ]),
                    ..Default::default()
                },
                "Llanowar Wastes" => Card {
                    name: "Llanowar Wastes".to_owned(),
                    card_type: CardType::Land,
                    produced_mana: HashMap::from([
                        (Mana::Black, 1),
                        (Mana::Green, 1),
                        (Mana::Colorless, 1),
                    ]),
                    ..Default::default()
                },
                "Brushland" => Card {
                    name: "Brushland".to_owned(),
                    card_type: CardType::Land,
                    produced_mana: HashMap::from([
                        (Mana::White, 1),
                        (Mana::Green, 1),
                        (Mana::Colorless, 1),
                    ]),
                    ..Default::default()
                },
                "Yavimaya Coast" => Card {
                    name: "Yavimaya Coast".to_owned(),
                    card_type: CardType::Land,
                    produced_mana: HashMap::from([
                        (Mana::Blue, 1),
                        (Mana::Green, 1),
                        (Mana::Colorless, 1),
                    ]),
                    ..Default::default()
                },
                "Caves of Koilos" => Card {
                    name: "Caves of Koilos".to_owned(),
                    card_type: CardType::Land,
                    produced_mana: HashMap::from([
                        (Mana::White, 1),
                        (Mana::Black, 1),
                        (Mana::Colorless, 1),
                    ]),
                    ..Default::default()
                },
                "Gemstone Mine" => Card {
                    name: "Gemstone Mine".to_owned(),
                    card_type: CardType::Land,
                    // TODO: max number of uses
                    produced_mana: HashMap::from([
                        (Mana::White, 1),
                        (Mana::Blue, 1),
                        (Mana::Black, 1),
                        (Mana::Red, 1),
                        (Mana::Green, 1),
                    ]),
                    ..Default::default()
                },
                "Reflecting Pool" => Card {
                    name: "Reflecting Pool".to_owned(),
                    card_type: CardType::Land,
                    // TODO: dynamically figure out what mana this produces
                    produced_mana: HashMap::from([
                        (Mana::White, 1),
                        (Mana::Blue, 1),
                        (Mana::Black, 1),
                        (Mana::Red, 1),
                        (Mana::Green, 1),
                    ]),
                    ..Default::default()
                },
                "Phyrexian Tower" => Card {
                    name: "Phyrexian Tower".to_owned(),
                    card_type: CardType::Land,
                    // TODO: the black mana from sac
                    produced_mana: HashMap::from([(Mana::Colorless, 1)]),
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
                "Ancient Tomb" => Card {
                    name: "Ancient Tomb".to_owned(),
                    card_type: CardType::Land,
                    produced_mana: HashMap::from([(Mana::Colorless, 2)]),
                    ..Default::default()
                },
                "Wall of Roots" => Card {
                    name: "Wall of Roots".to_owned(),
                    card_type: CardType::Creature,
                    produced_mana: HashMap::from([(Mana::Green, 1)]),
                    ..Default::default()
                },
                name => unimplemented!("{}", name),
            };

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

    pub fn put_bottom(&mut self, card: CardRef) {
        self.0.push_front(card)
    }

    pub fn iter(&self) -> Iter<'_, CardRef> {
        self.0.iter()
    }
}
