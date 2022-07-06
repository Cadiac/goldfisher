use std::collections::HashMap;

use crate::card::{Card};

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

pub fn can_pay_for(card: &Card, mana_sources: &[HashMap<Mana, usize>]) -> bool {
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
