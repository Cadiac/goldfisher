use crate::card::{CardRef, CardType, Zone};

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

pub fn is_creature(card: &&CardRef) -> bool {
    card.borrow().card_type == CardType::Creature
}

pub fn is_land(card: &&CardRef) -> bool {
    card.borrow().card_type == CardType::Land
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

pub fn is_mana_dork(card: &&CardRef) -> bool {
    let card = card.borrow();
    card.card_type == CardType::Creature && !card.produced_mana.is_empty()
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

pub fn is_named(card: &&CardRef, name: &str) -> bool {
    card.borrow().name == name
}

pub fn is_tapped(card: &&CardRef) -> bool {
    card.borrow().is_tapped
}

pub fn sort_by_produced_mana(a: &CardRef, b: &CardRef) -> std::cmp::Ordering {
    let a = a.borrow();
    let b = b.borrow();

    if a.produced_mana.len() == b.produced_mana.len() {
        // Try to save the mana sources with limited uses
        if a.remaining_uses.is_some() {
            return std::cmp::Ordering::Greater;
        }
        if b.remaining_uses.is_some() {
            return std::cmp::Ordering::Less;
        }
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
        .sum::<usize>()
        .partial_cmp(&b.borrow().cost.values().sum())
        .unwrap()
}
