use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::vec;

use crate::card::Card;

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

pub fn find_payment_for(
    card: &Card,
    mana_sources: &[Rc<RefCell<Card>>],
) -> Option<(Vec<Rc<RefCell<Card>>>, usize)> {
    if card.cost.is_empty() {
        return Some((vec![], 0));
    }

    let mut sources_to_pay_colors_with = HashMap::new();

    // Gather the color requirements first
    for color in &COLORS {
        if let Some(cost) = card.cost.get(color) {
            let available_sources: Vec<_> = mana_sources
                .iter()
                .flat_map(|source| {
                    source
                        .borrow()
                        .produced_mana
                        .get(color)
                        .and_then(|amount| Some((source, *amount)))
                })
                .collect();

            if *cost > available_sources.iter().map(|source| source.1).sum() {
                // Not enough mana to pay for this color
                return None;
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
            if used_sources.iter().any(|used| Rc::ptr_eq(used, source)) {
                continue;
            }

            paid += *amount;
            used_sources.push(Rc::clone(source));

            if paid >= *cost {
                floating += paid - cost;
                break;
            }
        }

        if paid < *cost {
            return None;
        }
    }

    if let Some(colorless_cost) = card.cost.get(&Mana::Colorless) {
        // Use the floating mana first
        let mut paid = floating;

        // If floating mana wasn't enough loop through remaining sources to pay for the colorless
        if paid < *colorless_cost {
            let remaining_sources = mana_sources
                .into_iter()
                .filter(|source| {
                    !used_sources.iter().any(|used| Rc::ptr_eq(used, source))
                })
                .collect::<Vec<_>>();

            for source in remaining_sources {
                let borrowed = source.borrow();
                if let Some(mana) = borrowed.produced_mana.values().max() {
                    paid += mana;
                    used_sources.push(Rc::clone(&source));
    
                    if paid >= *colorless_cost {
                        floating = paid - colorless_cost;
                        break;
                    }
                }
            }

            if paid < *colorless_cost {
                return None;
            }
        }
    }

    return Some((used_sources, floating));
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;

    #[test]
    fn it_finds_payment_no_mana_sources() {
        let card = Card { cost: HashMap::from([(Mana::Green, 1)]), ..Default::default() };

        let payment = find_payment_for(&card, &vec![]);
        assert_eq!(true, payment.is_none());
    }

    #[test]
    fn it_finds_payment_1cmc_right_color_basic() {
        let birds_of_paradise = Card { cost: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let forest = Card { produced_mana: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let forest_ref = Rc::new(RefCell::new(forest.clone()));

        let payment = find_payment_for(&birds_of_paradise, &vec![forest_ref.clone()]);
        assert_eq!(true, payment.is_some());
        let payment = payment.unwrap();
        assert_eq!(1, payment.0.len());
        assert_eq!(true, Rc::ptr_eq(&forest_ref, &payment.0[0]));
        assert_eq!(0, payment.1);
    }

    #[test]
    fn it_finds_payment_1cmc_wrong_color_basic() {
        let birds_of_paradise = Card { cost: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let mountain = Card { produced_mana: HashMap::from([(Mana::Red, 1)]), ..Default::default() };
        let mountain_ref = Rc::new(RefCell::new(mountain.clone()));

        let payment = find_payment_for(&birds_of_paradise, &vec![
            mountain_ref.clone()
        ]);
        assert_eq!(true, payment.is_none());
    }

    #[test]
    fn it_finds_payment_1cmc_multiple_basics() {
        let birds_of_paradise = Card { cost: HashMap::from([(Mana::Green, 1)]), ..Default::default() };

        let forest = Card { produced_mana: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let forest_ref = Rc::new(RefCell::new(forest.clone()));

        let mountain = Card { produced_mana: HashMap::from([(Mana::Red, 1)]), ..Default::default() };
        let mountain_ref = Rc::new(RefCell::new(mountain.clone()));

        let payment = find_payment_for(&birds_of_paradise, &vec![
            forest_ref.clone(),
            mountain_ref.clone()
        ]);
        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(1, payment.len());
        assert_eq!(true, Rc::ptr_eq(&forest_ref, &payment[0]));
        assert_eq!(0, floating);
    }

    #[test]
    fn it_finds_payment_1cmc_dual_land() {
        let birds_of_paradise = Card { cost: HashMap::from([(Mana::Green, 1)]), ..Default::default() };

        let taiga = Card { produced_mana: HashMap::from([(Mana::Green, 1), (Mana::Red, 1)]), ..Default::default() };
        let taiga_ref = Rc::new(RefCell::new(taiga.clone()));

        let payment = find_payment_for(&birds_of_paradise, &vec![
            taiga_ref.clone()
        ]);
        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(1, payment.len());
        assert_eq!(true, Rc::ptr_eq(&taiga_ref, &payment[0]));
        assert_eq!(0, floating);
    }

    /*
    #[test]
    fn it_checks_if_2cmc_colored_can_be_paid_for() {
        let card = Card {
            cost: HashMap::from([(Mana::Green, 2)]),
            ..Default::default()
        };

        assert_eq!(false, find_payment_for(&card, &vec![]));
        assert_eq!(false, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 1)])]));
        assert_eq!(true, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 2)])]));
        assert_eq!(false, find_payment_for(&card, &vec![HashMap::from([(Mana::Red, 2)])]));
        assert_eq!(false, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 1), (Mana::Red, 1)])]));
        assert_eq!(true, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 2), (Mana::Red, 2)])]));
        assert_eq!(true, find_payment_for(&card, &vec![HashMap::from([(Mana::Red, 2), (Mana::Green, 2)])]));
        assert_eq!(false, find_payment_for(&card, &vec![HashMap::from([(Mana::Red, 1)]), HashMap::from([(Mana::Green, 1)])]));
        assert_eq!(false, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 1)]), HashMap::from([(Mana::Green, 1)])]));
    }

    #[test]
    fn it_checks_if_2cmc_multicolored_can_be_paid_for() {
        let card = Card {
            cost: HashMap::from([(Mana::Green, 1), (Mana::Black, 1)]),
            ..Default::default()
        };

        assert_eq!(false, find_payment_for(&card, &vec![]));
        assert_eq!(false, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 1)])]));
        assert_eq!(false, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 2)])]));
        assert_eq!(false, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 1), (Mana::Black, 1)])]));
        assert_eq!(false, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 2), (Mana::Black, 2)])]));
        assert_eq!(true, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 1)]), HashMap::from([(Mana::Black, 1)])]));
        assert_eq!(true, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 1)]), HashMap::from([(Mana::Green, 1)]), HashMap::from([(Mana::Black, 1)])]));
    }

    #[test]
    fn it_checks_if_4cmc_colored_with_colorless_can_be_paid_for() {
        let card = Card {
            cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 3)]),
            ..Default::default()
        };

        assert_eq!(false, find_payment_for(&card, &vec![]));
        assert_eq!(false, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 1)])]));
        assert_eq!(false, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 2)])]));
        assert_eq!(false, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 1), (Mana::Black, 4)])]));
        assert_eq!(true, find_payment_for(&card, &vec![
            HashMap::from([(Mana::Green, 1)]),
            HashMap::from([(Mana::Red, 1)]),
            HashMap::from([(Mana::Black, 1)]),
            HashMap::from([(Mana::Blue, 1)])
        ]));
        assert_eq!(true, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 4)])]));
    }

    #[test]
    fn it_checks_if_colorless_can_be_paid_for() {
        let card = Card {
            cost: HashMap::from([(Mana::Colorless, 3)]),
            ..Default::default()
        };

        assert_eq!(false, find_payment_for(&card, &vec![]));
        assert_eq!(false, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 1)])]));
        assert_eq!(true, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 3)])]));
        assert_eq!(true, find_payment_for(&card, &vec![HashMap::from([(Mana::Green, 1), (Mana::Black, 4)])]));
        assert_eq!(true, find_payment_for(&card, &vec![
            HashMap::from([(Mana::Green, 1)]),
            HashMap::from([(Mana::Red, 1)]),
            HashMap::from([(Mana::Black, 1)]),
        ]));
        assert_eq!(true, find_payment_for(&card, &vec![
            HashMap::from([(Mana::Colorless, 1)]),
            HashMap::from([(Mana::Colorless, 2)]),
        ]));
    }
    */
}
