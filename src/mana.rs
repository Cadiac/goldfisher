use std::{collections::HashMap, hash::Hash};
use std::rc::Rc;
use std::vec;

use crate::card::{Card, CardRef};

pub type PaymentAndFloating = (Vec<CardRef>, HashMap<Mana, usize>);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
    mana_sources: &[CardRef],
    mut floating: HashMap<Mana, usize>
) -> Option<PaymentAndFloating> {
    if card.cost.is_empty() {
        return Some((vec![], HashMap::new()));
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
                        .map(|amount|(source, *amount))
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

    for (color, cost) in card.cost.iter() {
        if *color == Mana::Colorless {
            continue;
        }

        let mut paid = 0;

        // Try to spend any floating mana we might have
        let floating_mana = floating.entry(*color).or_insert(0);
        if *floating_mana < *cost {
            // Partial payment with floating mana
            paid += *floating_mana;
            *floating_mana = 0;
        } else {
            // Full colored payment with floating mnaa, continue to next color
            *floating_mana -= cost;
            continue;
        }

        let sources = sources_to_pay_colors_with.get(color).unwrap();
        for (source, amount) in sources.iter() {
            if used_sources.iter().any(|used| Rc::ptr_eq(used, source)) {
                continue;
            }

            paid += *amount;
            used_sources.push(Rc::clone(source));

            if paid >= *cost {
                *floating.entry(*color).or_insert(0) += paid - cost;
                break;
            }
        }

        if paid < *cost {
            return None;
        }
    }

    if let Some(cost) = card.cost.get(&Mana::Colorless) {
        // Use the floating mana first
        let mut paid = 0;

        // Try to spend any floating mana we might have
        for floating_mana in floating.values_mut() {
            // Try to spend any floating mana we might have
            if *floating_mana < *cost {
                // Partial payment with floating mana
                paid += *floating_mana;
                *floating_mana = 0;
            } else {
                // Full payment with floating mnaa, break the loop
                paid = *cost;
                *floating_mana -= cost;
                break;
            }
        }

        // If floating mana wasn't enough loop through remaining sources to pay for the colorless
        if paid < *cost {
            let mut remaining_sources = mana_sources
                .iter()
                .filter(|source| {
                    !used_sources.iter().any(|used| Rc::ptr_eq(used, source))
                })
                .collect::<Vec<_>>();

            remaining_sources.sort_by(|a, b| {
                b.borrow()
                    .produced_mana
                    .values()
                    .max()
                    .unwrap_or(&0)
                    .partial_cmp(a.borrow().produced_mana.values().max().unwrap_or(&0))
                    .unwrap()
            });

            for source in remaining_sources {
                let borrowed = source.borrow();
                let max_produced_mana_from_source = borrowed.produced_mana
                    .iter()
                    .max_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());

                if let Some((color, amount)) = max_produced_mana_from_source {
                    paid += amount;
                    used_sources.push(Rc::clone(source));
    
                    if paid >= *cost {
                        *floating.entry(*color).or_insert(0) = paid - cost;
                        break;
                    }
                }
            }

            if paid < *cost {
                return None;
            }
        }
    }

    Some((used_sources, floating))
}


#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;
    use std::cell::{RefCell};

    fn is_empty_mana_pool(floating: HashMap<Mana, usize>) -> bool {
        floating.values().all(|amount| *amount == 0)
    }

    #[test]
    fn it_finds_payment_no_mana_sources() {
        let card = Card { cost: HashMap::from([(Mana::Green, 1)]), ..Default::default() };

        let payment = find_payment_for(&card, &vec![], HashMap::new());
        assert_eq!(true, payment.is_none());
    }

    #[test]
    fn it_finds_payment_1cmc_right_color_basic() {
        let birds_of_paradise = Card { cost: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let forest = Card { produced_mana: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let forest_ref = Rc::new(RefCell::new(forest));

        let payment = find_payment_for(
            &birds_of_paradise,
            &vec![forest_ref.clone()],
            HashMap::new()
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(1, payment.len());
        assert_eq!(true, Rc::ptr_eq(&forest_ref, &payment[0]));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_1cmc_wrong_color_basic() {
        let birds_of_paradise = Card { cost: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let mountain = Card { produced_mana: HashMap::from([(Mana::Red, 1)]), ..Default::default() };
        let mountain_ref = Rc::new(RefCell::new(mountain));

        let payment = find_payment_for(
            &birds_of_paradise,
            &vec![mountain_ref.clone()],
            HashMap::new()
        );

        assert_eq!(true, payment.is_none());
    }

    #[test]
    fn it_finds_payment_1cmc_multiple_basics() {
        let birds_of_paradise = Card { cost: HashMap::from([(Mana::Green, 1)]), ..Default::default() };

        let forest = Card { produced_mana: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let forest_ref = Rc::new(RefCell::new(forest));

        let mountain = Card { produced_mana: HashMap::from([(Mana::Red, 1)]), ..Default::default() };
        let mountain_ref = Rc::new(RefCell::new(mountain));

        let payment = find_payment_for(
            &birds_of_paradise, 
            &vec![
                forest_ref.clone(),
                mountain_ref.clone(),
            ],
            HashMap::new()
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(1, payment.len());
        assert_eq!(true, Rc::ptr_eq(&forest_ref, &payment[0]));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_1cmc_dual_land() {
        let birds_of_paradise = Card { cost: HashMap::from([(Mana::Green, 1)]), ..Default::default() };

        let taiga = Card { produced_mana: HashMap::from([(Mana::Green, 1), (Mana::Red, 1)]), ..Default::default() };
        let taiga_ref = Rc::new(RefCell::new(taiga));

        let payment = find_payment_for(
            &birds_of_paradise, 
            &vec![taiga_ref.clone()],
            HashMap::new(),
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(1, payment.len());
        assert_eq!(true, Rc::ptr_eq(&taiga_ref, &payment[0]));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_1cmc_excess_mana() {
        let birds_of_paradise = Card { cost: HashMap::from([(Mana::Green, 1)]), ..Default::default() };

        let gaeas_cradle  = Card { produced_mana: HashMap::from([(Mana::Green, 2)]), ..Default::default() };
        let gaeas_cradle_ref = Rc::new(RefCell::new(gaeas_cradle));

        let payment = find_payment_for(
            &birds_of_paradise,
            &vec![gaeas_cradle_ref.clone()],
            HashMap::new(),
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(1, payment.len());
        assert_eq!(true, Rc::ptr_eq(&gaeas_cradle_ref, &payment[0]));
        assert_eq!(1, floating.len());
        assert_eq!(1, *floating.get(&Mana::Green).unwrap());
    }

    #[test]
    fn it_finds_payment_2cmc_right_colors() {
        let channel = Card { cost: HashMap::from([(Mana::Green, 2)]), ..Default::default() };

        let forest = Card { produced_mana: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let forest_1 = Rc::new(RefCell::new(forest.clone()));
        let forest_2 = Rc::new(RefCell::new(forest.clone()));
        let forest_3 = Rc::new(RefCell::new(forest.clone()));

        let payment = find_payment_for(
            &channel,
            &vec![
                forest_1.clone(),
                forest_2.clone(),
                forest_3.clone(),
            ],
            HashMap::new(),
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(2, payment.len());
        assert_eq!(true, Rc::ptr_eq(&forest_1, &payment[0]));
        assert_eq!(true, Rc::ptr_eq(&forest_2, &payment[1]));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_3cmc_multicolor() {
        let jungle_troll = Card {
            cost: HashMap::from([(Mana::Green, 1), (Mana::Red, 1), (Mana::Colorless, 1)]),
            ..Default::default()
        };

        let forest = Card { produced_mana: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let mountain = Card { produced_mana: HashMap::from([(Mana::Red, 1)]), ..Default::default() };
        let forest_1 = Rc::new(RefCell::new(forest.clone()));
        let forest_2 = Rc::new(RefCell::new(forest.clone()));
        let mountain_1 = Rc::new(RefCell::new(mountain.clone()));

        let payment = find_payment_for(
            &jungle_troll,
            &vec![
                forest_1.clone(),
                forest_2.clone(),
                mountain_1.clone(),
            ],
            HashMap::new(),
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(3, payment.len());
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&forest_1, source)));
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&forest_2, source)));
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&mountain_1, source)));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_3cmc_colorless() {
        let metalworker = Card {
            cost: HashMap::from([(Mana::Colorless, 3)]),
            ..Default::default()
        };

        let forest = Card { produced_mana: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let mountain = Card { produced_mana: HashMap::from([(Mana::Red, 1)]), ..Default::default() };
        let forest_1 = Rc::new(RefCell::new(forest.clone()));
        let forest_2 = Rc::new(RefCell::new(forest.clone()));
        let mountain_1 = Rc::new(RefCell::new(mountain.clone()));

        let payment = find_payment_for(
            &metalworker,
            &vec![
                forest_1.clone(),
                forest_2.clone(),
                mountain_1.clone(),
            ],
            HashMap::new(),
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(3, payment.len());
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&forest_1, source)));
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&forest_2, source)));
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&mountain_1, source)));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_3cmc_colorless_prefers_sol_lands() {
        let metalworker = Card {
            cost: HashMap::from([(Mana::Colorless, 3)]),
            ..Default::default()
        };

        let forest = Card { produced_mana: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let ancient_tomb = Card { produced_mana: HashMap::from([(Mana::Colorless, 2)]), ..Default::default() };
        let forest_1 = Rc::new(RefCell::new(forest.clone()));
        let forest_2 = Rc::new(RefCell::new(forest.clone()));
        let ancient_tomb_1 = Rc::new(RefCell::new(ancient_tomb.clone()));

        let payment = find_payment_for(
            &metalworker,
            &vec![
                forest_1.clone(),
                forest_2.clone(),
                ancient_tomb_1.clone(),
            ],
            HashMap::new(),
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(2, payment.len());
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&forest_1, source)));
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&ancient_tomb_1, source)));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_3cmc_saves_colors() {
        let jungle_troll = Card {
            cost: HashMap::from([(Mana::Green, 1), (Mana::Red, 1), (Mana::Colorless, 1)]),
            ..Default::default()
        };

        let forest = Card { produced_mana: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let mountain = Card { produced_mana: HashMap::from([(Mana::Red, 1)]), ..Default::default() };
        let city_of_brass = Card {
            produced_mana: HashMap::from([
                (Mana::White, 1),
                (Mana::Blue, 1),
                (Mana::Black, 1),
                (Mana::Green, 1),
                (Mana::Red, 1),
            ]),
            ..Default::default()
        };
        let taiga = Card {
            produced_mana: HashMap::from([
                (Mana::Green, 1),
                (Mana::Red, 1),
            ]),
            ..Default::default()
        };
        let forest_1 = Rc::new(RefCell::new(forest.clone()));
        let forest_2 = Rc::new(RefCell::new(forest.clone()));
        let mountain_1 = Rc::new(RefCell::new(mountain.clone()));
        let taiga_1 = Rc::new(RefCell::new(taiga.clone()));
        let city_of_brass_1 = Rc::new(RefCell::new(city_of_brass.clone()));
        let city_of_brass_2 = Rc::new(RefCell::new(city_of_brass.clone()));

        // Note: These must be provided in ascending order by mana produced or else this won't work
        let payment = find_payment_for(
            &jungle_troll,
            &vec![
                forest_1.clone(),
                forest_2.clone(),
                mountain_1.clone(),
                taiga_1.clone(),
                city_of_brass_1.clone(),
                city_of_brass_2.clone(),
            ],
            HashMap::new()
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(3, payment.len());
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&forest_1, source)));
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&forest_2, source)));
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&mountain_1, source)));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_1cmc_exact_floating_mana() {
        let birds_of_paradise = Card { cost: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let forest = Card { produced_mana: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let forest_ref = Rc::new(RefCell::new(forest));

        let payment = find_payment_for(
            &birds_of_paradise,
            &vec![forest_ref.clone()],
            HashMap::from([(Mana::Green, 1)])
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(true, payment.is_empty());
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_1cmc_execss_floating_mana() {
        let birds_of_paradise = Card { cost: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let forest = Card { produced_mana: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let forest_ref = Rc::new(RefCell::new(forest));

        let payment = find_payment_for(
            &birds_of_paradise,
            &vec![forest_ref.clone()],
            HashMap::from([(Mana::Green, 2), (Mana::Red, 1)])
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(true, payment.is_empty());
        assert_eq!(1, *floating.get(&Mana::Green).unwrap());
        assert_eq!(1, *floating.get(&Mana::Red).unwrap());
    }

    #[test]
    fn it_finds_payment_2cmc_floating_mana_for_colorless() {
        let grizzly_bear = Card { cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 1)]), ..Default::default() };
        let forest = Card { produced_mana: HashMap::from([(Mana::Green, 1)]), ..Default::default() };
        let forest_ref = Rc::new(RefCell::new(forest));

        let payment = find_payment_for(
            &grizzly_bear,
            &vec![forest_ref.clone()],
            HashMap::from([(Mana::Red, 2)])
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(1, payment.len());
        assert_eq!(true, Rc::ptr_eq(&forest_ref, &payment[0]));
        assert_eq!(1, *floating.get(&Mana::Red).unwrap());
        assert_eq!(0, *floating.get(&Mana::Green).unwrap());
    }
}
