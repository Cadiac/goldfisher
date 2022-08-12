use std::{collections::HashMap, hash::Hash};
use std::rc::Rc;
use std::vec;

use crate::card::{CardRef, CardType};

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
    card: CardRef,
    mana_sources: &[CardRef],
    mut floating: HashMap<Mana, usize>,
    is_aluren: bool,
) -> Option<PaymentAndFloating> {
    if card.borrow().cost.is_empty() {
        return Some((vec![], floating));
    }

    if is_aluren && card.borrow().card_type == CardType::Creature && card.borrow().cost.values().sum::<usize>() <= 3 {
        return Some((vec![], floating));
    }

    let mut sources_to_pay_colors_with = HashMap::new();

    // Gather the color requirements first
    for color in &COLORS {
        if let Some(cost) = card.borrow().cost.get(color) {
            let available_sources: Vec<_> = mana_sources
                .iter()
                // Prevent doing something like paying for Elvish Spirit Guide with itself
                .filter(|source| !Rc::ptr_eq(*source, &card))
                .flat_map(|source| {
                    source
                        .borrow()
                        .produced_mana
                        .get(color)
                        .map(|amount|(source, *amount))
                })
                .collect();

            let total_available: usize = available_sources.iter().map(|source| source.1).sum();
            let total_floating = floating.get(color).unwrap_or(&0);
            if *cost > total_available + total_floating {
                // Not enough mana to pay for this color
                return None;
            }

            sources_to_pay_colors_with.insert(color, available_sources);
        }
    }

    let mut used_sources = Vec::new();

    for (color, cost) in card.borrow().cost.iter() {
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

    if let Some(cost) = card.borrow().cost.get(&Mana::Colorless) {
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
    use crate::card::{Card};

    fn is_empty_mana_pool(floating: HashMap<Mana, usize>) -> bool {
        floating.values().all(|amount| *amount == 0)
    }

    #[test]
    fn it_finds_payment_no_mana_sources() {
        let card = Card::new_as_ref("Birds of Paradise");

        let payment = find_payment_for(card, &vec![], HashMap::new(), false);
        assert_eq!(true, payment.is_none());
    }

    #[test]
    fn it_finds_payment_1cmc_right_color_basic() {
        let birds_of_paradise = Card::new_as_ref("Birds of Paradise");
        let forest = Card::new_as_ref("Forest");

        let payment = find_payment_for(
            birds_of_paradise,
            &vec![forest.clone()],
            HashMap::new(),
            false
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(1, payment.len());
        assert_eq!(true, Rc::ptr_eq(&forest, &payment[0]));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_1cmc_wrong_color_basic() {
        let birds_of_paradise = Card::new_as_ref("Birds of Paradise");
        let mountain = Card::new_as_ref("Mountain");

        let payment = find_payment_for(
            birds_of_paradise,
            &vec![mountain.clone()],
            HashMap::new(),
            false
        );

        assert_eq!(true, payment.is_none());
    }

    #[test]
    fn it_finds_payment_1cmc_multiple_basics() {
        let birds_of_paradise = Card::new_as_ref("Birds of Paradise");
        let forest = Card::new_as_ref("Forest");
        let mountain = Card::new_as_ref("Mountain");

        let payment = find_payment_for(
            birds_of_paradise,
            &vec![
                forest.clone(),
                mountain.clone(),
            ],
            HashMap::new(),
            false,
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(1, payment.len());
        assert_eq!(true, Rc::ptr_eq(&forest, &payment[0]));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_1cmc_dual_land() {
        let birds_of_paradise = Card::new_as_ref("Birds of Paradise");
        let taiga = Card::new_as_ref("Taiga");

        let payment = find_payment_for(
            birds_of_paradise, 
            &vec![taiga.clone()],
            HashMap::new(),
            false,
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(1, payment.len());
        assert_eq!(true, Rc::ptr_eq(&taiga, &payment[0]));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_1cmc_excess_mana() {
        let birds_of_paradise = Card::new_as_ref("Birds of Paradise");
        let hickory_woodlot = Card::new_as_ref("Hickory Woodlot");
        hickory_woodlot.borrow_mut().is_tapped = false;

        let payment = find_payment_for(
            birds_of_paradise,
            &vec![hickory_woodlot.clone()],
            HashMap::new(),
            false,
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(1, payment.len());
        assert_eq!(true, Rc::ptr_eq(&hickory_woodlot, &payment[0]));
        assert_eq!(1, floating.len());
        assert_eq!(1, *floating.get(&Mana::Green).unwrap());
    }

    #[test]
    fn it_finds_payment_2cmc_right_colors() {
        let rofellos = Card::new_as_ref("Rofellos, Llanowar Emissary");
        let forest_1 = Card::new_as_ref("Forest");
        let forest_2 = Card::new_as_ref("Forest");
        let forest_3 = Card::new_as_ref("Forest");

        let payment = find_payment_for(
            rofellos,
            &vec![
                forest_1.clone(),
                forest_2.clone(),
                forest_3.clone(),
            ],
            HashMap::new(),
            false,
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(2, payment.len());
        assert_eq!(true, Rc::ptr_eq(&forest_1, &payment[0]));
        assert_eq!(true, Rc::ptr_eq(&forest_2, &payment[1]));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_2cmc_multicolor() {
        let eladamris_call = Card::new_as_ref("Eladamri's Call");

        let forest = Card::new_as_ref("Forest");
        let plains = Card::new_as_ref("Plains");
        let mountain = Card::new_as_ref("Mountain");

        let payment = find_payment_for(
            eladamris_call,
            &vec![
                forest.clone(),
                plains.clone(),
                mountain.clone(),
            ],
            HashMap::new(),
            false,
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(2, payment.len());
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&forest, source)));
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&plains, source)));
        assert_eq!(false, payment.iter().any(|source| Rc::ptr_eq(&mountain, source)));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_3cmc_multicolor() {
        let vindicate = Card::new_as_ref("Vindicate");

        let plains = Card::new_as_ref("Plains");
        let swamp = Card::new_as_ref("Swamp");
        let mountain = Card::new_as_ref("Mountain");

        let payment = find_payment_for(
            vindicate,
            &vec![
                plains.clone(),
                swamp.clone(),
                mountain.clone()
            ],
            HashMap::new(),
            false,
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(3, payment.len());
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&plains, source)));
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&swamp, source)));
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&mountain, source)));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_2cmc_colorless() {
        let altar_of_dementia = Card::new_as_ref("Altar of Dementia");

        let forest = Card::new_as_ref("Forest");
        let mountain = Card::new_as_ref("Mountain");

        let payment = find_payment_for(
            altar_of_dementia,
            &vec![
                forest.clone(),
                mountain.clone()
            ],
            HashMap::new(),
            false,
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(2, payment.len());
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&forest, source)));
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&mountain, source)));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_2cmc_colorless_prefers_sol_lands() {
        let altar_of_dementia = Card::new_as_ref("Altar of Dementia");

        let forest = Card::new_as_ref("Forest");
        let mountain = Card::new_as_ref("Mountain");
        let ancient_tomb = Card::new_as_ref("Ancient Tomb");

        let payment = find_payment_for(
            altar_of_dementia,
            &vec![
                forest.clone(),
                mountain.clone(),
                ancient_tomb.clone()
            ],
            HashMap::new(),
            false,
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(1, payment.len());
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&ancient_tomb, source)));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_3cmc_saves_colors() {
        let vindicate = Card::new_as_ref("Vindicate");

        let plains_1 = Card::new_as_ref("Plains");
        let plains_2 = Card::new_as_ref("Plains");
        let swamp = Card::new_as_ref("Swamp");
        let city_of_brass_1 = Card::new_as_ref("City of Brass");
        let city_of_brass_2 = Card::new_as_ref("City of Brass");
        let scrubland = Card::new_as_ref("Scrubland");

        // Note: These must be provided in ascending order by mana produced or else this won't work
        let payment = find_payment_for(
            vindicate,
            &vec![
                plains_1.clone(),
                plains_2.clone(),
                swamp.clone(),
                scrubland.clone(),
                city_of_brass_1.clone(),
                city_of_brass_2.clone(),
            ],
            HashMap::new(),
            false,
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(3, payment.len());
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&plains_1, source)));
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&plains_2, source)));
        assert_eq!(true, payment.iter().any(|source| Rc::ptr_eq(&swamp, source)));
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_1cmc_exact_floating_mana() {
        let birds_of_paradise = Card::new_as_ref("Birds of Paradise");
        let forest = Card::new_as_ref("Forest");

        let payment = find_payment_for(
            birds_of_paradise,
            &vec![forest.clone()],
            HashMap::from([(Mana::Green, 1)]),
            false,
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(true, payment.is_empty());
        assert_eq!(true, is_empty_mana_pool(floating));
    }

    #[test]
    fn it_finds_payment_1cmc_execss_floating_mana() {
        let birds_of_paradise = Card::new_as_ref("Birds of Paradise");
        let forest = Card::new_as_ref("Forest");

        let payment = find_payment_for(
            birds_of_paradise,
            &vec![forest.clone()],
            HashMap::from([(Mana::Green, 2), (Mana::Red, 1)]),
            false,
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(true, payment.is_empty());
        assert_eq!(1, *floating.get(&Mana::Green).unwrap());
        assert_eq!(1, *floating.get(&Mana::Red).unwrap());
    }

    #[test]
    fn it_finds_payment_2cmc_floating_mana_for_colorless() {
        let wall_of_roots = Card::new_as_ref("Wall of Roots");
        let forest = Card::new_as_ref("Forest");

        let payment = find_payment_for(
            wall_of_roots,
            &vec![forest.clone()],
            HashMap::from([(Mana::Red, 2)]),
            false,
        );

        assert_eq!(true, payment.is_some());
        let (payment, floating) = payment.unwrap();
        assert_eq!(1, payment.len());
        assert_eq!(true, Rc::ptr_eq(&forest, &payment[0]));
        assert_eq!(1, *floating.get(&Mana::Red).unwrap());
        assert_eq!(0, *floating.get(&Mana::Green).unwrap());
    }
}
