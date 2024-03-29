use crate::{
    card::{CardRef, CardType, SearchFilter, SubType, Zone},
    game::Game,
    mana::Mana,
};
use std::collections::HashMap;

pub fn is_battlefield(card: &&CardRef) -> bool {
    card.borrow().zone == Zone::Battlefield
}

pub fn is_hand(card: &&CardRef) -> bool {
    card.borrow().zone == Zone::Hand
}

pub fn is_graveyard(card: &&CardRef) -> bool {
    card.borrow().zone == Zone::Graveyard
}

pub fn is_exile(card: &&CardRef) -> bool {
    card.borrow().zone == Zone::Exile
}

pub fn is_library(card: &&CardRef) -> bool {
    card.borrow().zone == Zone::Library
}

pub fn is_rector(card: &&CardRef) -> bool {
    card.borrow().name == "Academy Rector"
}

pub fn is_pattern(card: &&CardRef) -> bool {
    card.borrow().name == "Pattern of Rebirth"
}

pub fn is_sac_outlet(card: &&CardRef) -> bool {
    card.borrow().is_sac_outlet
}

pub fn is_basic(card: &&CardRef) -> bool {
    ["Plains", "Island", "Swamp", "Mountain", "Forest"].contains(&card.borrow().name.as_str())
}

pub fn is_mana_dork(card: &&CardRef) -> bool {
    let card = card.borrow();
    card.card_types.contains(&CardType::Creature) && !card.produced_mana.is_empty()
}

pub fn is_mana_source(card: &&CardRef) -> bool {
    !card.borrow().produced_mana.is_empty()
}

pub fn is_single_use_mana(card: &&CardRef) -> bool {
    match card.borrow().remaining_uses {
        Some(uses) => uses == 1,
        None => false,
    }
}

pub fn is_card_type(card: &&CardRef, card_type: &CardType) -> bool {
    card.borrow().card_types.contains(card_type)
}

pub fn is_sub_type(card: &&CardRef, sub_type: &SubType) -> bool {
    card.borrow().sub_types.contains(sub_type)
}

pub fn is_cost_reducer(card: &&CardRef) -> bool {
    card.borrow().cost_reduction.is_some()
}

pub fn is_color(card: &&CardRef, color: Mana) -> bool {
    match card.borrow().cost.get(&color) {
        Some(cost) => *cost > 0,
        None => false,
    }
}

pub fn is_zone(card: &&CardRef, zone: &Zone) -> bool {
    card.borrow().zone == *zone
}

pub fn is_named(card: &&CardRef, name: &str) -> bool {
    card.borrow().name == name
}

pub fn is_tapped(card: &&CardRef) -> bool {
    card.borrow().is_tapped
}

pub fn sort_by_best_mana_to_play(a: &CardRef, b: &CardRef) -> std::cmp::Ordering {
    let a = a.borrow();
    let b = b.borrow();

    if a.produced_mana.len() == b.produced_mana.len() {
        // Play the mana source with most uses
        return a
            .remaining_uses
            .unwrap_or(usize::MAX)
            .partial_cmp(&b.remaining_uses.unwrap_or(usize::MAX))
            .unwrap();
    }

    a.produced_mana
        .len()
        .partial_cmp(&b.produced_mana.len())
        .unwrap()
}

pub fn sort_by_best_mana_to_use(a: &CardRef, b: &CardRef) -> std::cmp::Ordering {
    let a = a.borrow();
    let b = b.borrow();

    if a.produced_mana.len() == b.produced_mana.len() {
        // Try to save the mana sources with least uses
        return b
            .remaining_uses
            .unwrap_or(usize::MAX)
            .partial_cmp(&a.remaining_uses.unwrap_or(usize::MAX))
            .unwrap();
    }

    a.produced_mana
        .len()
        .partial_cmp(&b.produced_mana.len())
        .unwrap()
}

pub fn sort_by_cmc(a: &CardRef, b: &CardRef) -> std::cmp::Ordering {
    a.borrow()
        .cost
        .values()
        .sum::<i32>()
        .partial_cmp(&b.borrow().cost.values().sum())
        .unwrap()
}

pub fn is_empty_mana_pool(floating: HashMap<Mana, u32>) -> bool {
    floating.values().all(|amount| *amount == 0)
}

pub fn count_in_hand(game: &Game, names: &[&str]) -> usize {
    game.game_objects
        .iter()
        .filter(|card| is_hand(&card) && names.iter().any(|name| is_named(&card, name)))
        .count()
}

// TODO: Make these composable, and use them like `vec![Library, Creature]` etc
pub fn apply_search_filter(game: &Game, search_filter: &Option<SearchFilter>) -> Vec<CardRef> {
    match search_filter {
        Some(SearchFilter::Creature) => game
            .game_objects
            .iter()
            .filter(|card| is_library(card) && is_card_type(card, &CardType::Creature))
            .cloned()
            .collect(),
        Some(SearchFilter::EnchantmentArtifact) => game
            .game_objects
            .iter()
            .filter(|card| {
                is_library(card)
                    && (is_card_type(card, &CardType::Enchantment)
                        || is_card_type(card, &CardType::Artifact))
            })
            .cloned()
            .collect(),
        Some(SearchFilter::Wish(card_types)) => game
            .deck
            .sideboard
            .iter()
            .filter(|card| {
                card_types
                    .iter()
                    .any(|card_type| is_card_type(card, card_type))
            })
            .cloned()
            .collect(),
        Some(SearchFilter::BlueInstant) => game
            .game_objects
            .iter()
            .filter(|card| {
                is_library(card)
                    && is_card_type(card, &CardType::Instant)
                    && is_color(card, Mana::Blue)
            })
            .cloned()
            .collect(),
        Some(SearchFilter::GreenCreature) => game
            .game_objects
            .iter()
            .filter(|card| {
                is_library(card)
                    && is_card_type(card, &CardType::Creature)
                    && is_color(card, Mana::Green)
            })
            .cloned()
            .collect(),
        Some(SearchFilter::Blue) => game
            .game_objects
            .iter()
            .filter(|card| is_color(card, Mana::Blue))
            .cloned()
            .collect(),
        Some(SearchFilter::Land(land_types)) => game
            .game_objects
            .iter()
            .filter(|card| {
                is_library(card)
                    && is_card_type(card, &CardType::Land)
                    && land_types
                        .iter()
                        .any(|land_type| is_sub_type(card, &SubType::Land(land_type.clone())))
            })
            .cloned()
            .collect(),
        None => game
            .game_objects
            .iter()
            .filter(is_library)
            .cloned()
            .collect(),
    }
}

pub fn group_by_name(game_objects: Vec<CardRef>) -> HashMap<String, Vec<CardRef>> {
    let mut cards = HashMap::new();

    for game_object in game_objects.iter() {
        let name = &game_object.borrow().name;
        cards
            .entry(name.clone())
            .or_insert(vec![])
            .push(game_object.clone());
    }

    cards
}

pub fn find_named(cards: &HashMap<String, Vec<CardRef>>, name: &str) -> Option<CardRef> {
    cards.get(name).and_then(|copies| copies.first()).cloned()
}

pub fn find_n_with_priority(game: &Game, count: usize, priority_list: &[&str]) -> Vec<CardRef> {
    let mut found = Vec::with_capacity(count);

    for card_name in priority_list {
        let to_find = count - found.len();

        for card in game
            .game_objects
            .iter()
            .filter(|card| is_library(card) && card.borrow().name == *card_name)
            .take(to_find)
        {
            // Place these outside the game temporarily
            card.borrow_mut().zone = Zone::Outside;
            found.push(card.clone());
        }
    }

    if found.len() < count {
        // Find anything from library to meet the desired count the best we can
        found.extend(
            game.game_objects
                .iter()
                .filter(is_library)
                .take(count - found.len())
                .cloned(),
        );
    }

    found
}
