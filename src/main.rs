use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq)]
pub enum CardType {
    Creature,
    Enchantment,
    Sorcery,
    Land,
}

impl Default for CardType {
    fn default() -> Self {
        CardType::Creature
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mana {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

const COLORS: [Mana; 5] = [Mana::White, Mana::Blue, Mana::Black, Mana::Red, Mana::Green];

#[derive(Clone, Debug, Default)]
pub struct Card {
    name: String,
    card_type: CardType,
    cost: HashMap<Mana, usize>,
    produced_mana: HashMap<Mana, usize>,
    is_sac_outlet: bool,
    is_rector: bool,
    is_pattern: bool,
    is_summoning_sick: bool,
    is_tapped: bool,
}

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

fn create_deck(cards: Vec<(&str, usize)>) -> Deck {
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

fn main() {
    let mut deck = create_deck(vec![
        ("Birds of Paradise", 4),
        ("Llanowar Elves", 2),
        ("Carrion Feeder", 4),
        ("Nantuko Husk", 3),
        ("Phyrexian Ghoul", 1),
        ("Pattern of Rebirth", 4),
        ("Academy Rector", 4),
        ("Forest", 12),
        ("Swamp", 4),
        ("Plains", 4),
    ]);

    deck.shuffle();

    let mut turn = 0;
    let mut hand = Vec::with_capacity(8);
    let mut battlefield: Vec<Card> = Vec::new();

    let is_first_player = true;

    // Take the opening 7
    for _ in 0..7 {
        if let Some(card) = deck.draw() {
            hand.push(card)
        }
    }

    loop {
        for card in battlefield.iter_mut() {
            card.is_summoning_sick = false;
        }

        // 0. Draw a card for the turn
        if turn > 0 || !is_first_player {
            if let Some(card) = deck.draw() {
                hand.push(card)
            }
        }

        // 1. Play a land card, preferring lands with most colors produced
        let mut lands_in_hand = hand
            .iter()
            .filter(|card| card.card_type == CardType::Land)
            .cloned()
            .collect::<Vec<_>>();
        lands_in_hand.sort_by(|a, b| {
            a.produced_mana
                .len()
                .partial_cmp(&b.produced_mana.len())
                .unwrap()
        });
        // Play the one that produces most colors
        // TODO: Play the one that produces most cards that can be played
        if let Some(land) = lands_in_hand.pop() {
            battlefield.push(land);
        }

        // 2. Figure out which cards in our hand we can pay for
        let nonlands_in_hand = hand
            .iter()
            .filter(|card| card.card_type != CardType::Land)
            .cloned();

        let mana_sources = battlefield.iter().filter(|card| {
            !card.produced_mana.is_empty() && !card.is_summoning_sick && !card.is_tapped
        });

        let mut available_mana: Vec<_> = mana_sources
            .map(|card| card.produced_mana.clone())
            .collect();
        available_mana.sort_by(|a, b| a.len().partial_cmp(&b.len()).unwrap());

        let castable = nonlands_in_hand.filter(|card| can_pay_for(card, &available_mana));

        // N. Do we have it?

        // If not, take another turn

        turn += 1;
    }
}

fn can_pay_for(card: &Card, mana_sources: &[HashMap<Mana, usize>]) -> bool {
    if card.cost.is_empty() {
        return true;
    }

    let mut sources_to_pay_colors_with = HashMap::new();

    // Gather the color requirements first
    for color in &COLORS {
        if let Some(cost) = card.cost.get(color) {
            let available_sources: Vec<_> = mana_sources
                .iter()
                .flat_map(|source| source.get(color).and_then(|amount| Some((source, amount))))
                .collect();

            if *cost > available_sources.iter().map(|source| source.1).sum() {
                // Not enough mana to pay for this color
                return false;
            }

            sources_to_pay_colors_with.insert(color, available_sources);
        }
    }

    let mut used_sources = Vec::new();

    let mut floating = 0;

    for (color, cost) in card.cost.iter() {
        if *color == Mana::Colorless {
            continue;
        }

        let mut paid = 0;

        let sources = sources_to_pay_colors_with.get(color).unwrap();
        for (source, amount) in sources.iter() {
            if used_sources.contains(source) {
                continue;
            }

            paid += *amount;
            used_sources.push(source);

            if paid >= *cost {
                floating += paid - cost;
                break;
            }
        }

        if paid < *cost {
            return false;
        }
    }

    if let Some(colorless_cost) = card.cost.get(&Mana::Colorless) {
        let colorless_available: usize = mana_sources
            .iter()
            .filter(|source| !used_sources.contains(source))
            .map(|source| source.values().max().unwrap_or(&0))
            .sum();

        return floating + colorless_available >= *colorless_cost;
    }

    return true;
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;

    #[test]
    fn it_checks_if_1cmc_colored_can_be_paid_for() {
        let card = Card {
            cost: HashMap::from([(Mana::Green, 1)]),
            ..Default::default()
        };

        assert_eq!(false, can_pay_for(&card, &vec![]));
        assert_eq!(true, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 1)])]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Red, 1)])]));
        assert_eq!(true, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 1), (Mana::Red, 1)])]));
        assert_eq!(true, can_pay_for(&card, &vec![HashMap::from([(Mana::Red, 1), (Mana::Green, 1)])]));
        assert_eq!(true, can_pay_for(&card, &vec![HashMap::from([(Mana::Red, 1)]), HashMap::from([(Mana::Green, 1)])]));
    }

    #[test]
    fn it_checks_if_2cmc_colored_can_be_paid_for() {
        let card = Card {
            cost: HashMap::from([(Mana::Green, 2)]),
            ..Default::default()
        };

        assert_eq!(false, can_pay_for(&card, &vec![]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 1)])]));
        assert_eq!(true, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 2)])]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Red, 2)])]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 1), (Mana::Red, 1)])]));
        assert_eq!(true, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 2), (Mana::Red, 2)])]));
        assert_eq!(true, can_pay_for(&card, &vec![HashMap::from([(Mana::Red, 2), (Mana::Green, 2)])]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Red, 1)]), HashMap::from([(Mana::Green, 1)])]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 1)]), HashMap::from([(Mana::Green, 1)])]));
    }

    #[test]
    fn it_checks_if_2cmc_multicolored_can_be_paid_for() {
        let card = Card {
            cost: HashMap::from([(Mana::Green, 1), (Mana::Black, 1)]),
            ..Default::default()
        };

        assert_eq!(false, can_pay_for(&card, &vec![]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 1)])]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 2)])]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 1), (Mana::Black, 1)])]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 2), (Mana::Black, 2)])]));
        assert_eq!(true, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 1)]), HashMap::from([(Mana::Black, 1)])]));
        assert_eq!(true, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 1)]), HashMap::from([(Mana::Green, 1)]), HashMap::from([(Mana::Black, 1)])]));
    }

    #[test]
    fn it_checks_if_4cmc_colored_with_colorless_can_be_paid_for() {
        let card = Card {
            cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 3)]),
            ..Default::default()
        };

        assert_eq!(false, can_pay_for(&card, &vec![]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 1)])]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 2)])]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 1), (Mana::Black, 4)])]));
        assert_eq!(true, can_pay_for(&card, &vec![
            HashMap::from([(Mana::Green, 1)]),
            HashMap::from([(Mana::Red, 1)]),
            HashMap::from([(Mana::Black, 1)]),
            HashMap::from([(Mana::Blue, 1)])
        ]));
        assert_eq!(true, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 4)])]));
    }

    #[test]
    fn it_checks_if_colorless_can_be_paid_for() {
        let card = Card {
            cost: HashMap::from([(Mana::Colorless, 3)]),
            ..Default::default()
        };

        assert_eq!(false, can_pay_for(&card, &vec![]));
        assert_eq!(false, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 1)])]));
        assert_eq!(true, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 3)])]));
        assert_eq!(true, can_pay_for(&card, &vec![HashMap::from([(Mana::Green, 1), (Mana::Black, 4)])]));
        assert_eq!(true, can_pay_for(&card, &vec![
            HashMap::from([(Mana::Green, 1)]),
            HashMap::from([(Mana::Red, 1)]),
            HashMap::from([(Mana::Black, 1)]),
        ]));
        assert_eq!(true, can_pay_for(&card, &vec![
            HashMap::from([(Mana::Colorless, 1)]),
            HashMap::from([(Mana::Colorless, 2)]),
        ]));
    }
}
