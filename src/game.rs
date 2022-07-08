use std::cell::RefCell;
use std::rc::Rc;

use crate::card::{Card, CardType, Zone};
use crate::deck::{Deck};
use crate::mana::find_payment_for;

pub fn is_combo_ready(game_objects: &Vec<Rc<RefCell<Card>>>) -> bool {
    let have_sac_outlet = game_objects.iter().any(|card| {
        let card = card.borrow();
        card.zone == Zone::Battlefield && card.is_sac_outlet
    });
    let have_pattern_attached = game_objects.iter().any(|card| {
        let card = card.borrow();
        card.zone == Zone::Battlefield && card.is_pattern
    });

    have_sac_outlet && have_pattern_attached
}

pub fn cast_creatures(game_objects: &Vec<Rc<RefCell<Card>>>) {
    let castable = find_castable(game_objects);

    let creature = castable
        .iter()
        .find(|(card, _)| card.borrow().card_type == CardType::Creature);
    if let Some((card_ref, payment)) = creature {
        cast_spell(card_ref, payment.as_ref().unwrap(), None);
    }
}

pub fn cast_sac_outlets(game_objects: &Vec<Rc<RefCell<Card>>>) {
    let castable = find_castable(game_objects);

    let sac_creature = castable.iter().find(|(card, _)| {
        let card = card.borrow();
        card.card_type == CardType::Creature && card.is_sac_outlet
    });
    if let Some((card_ref, payment)) = sac_creature {
        cast_spell(card_ref, payment.as_ref().unwrap(), None);
    }
}

pub fn cast_pattern_of_rebirths(game_objects: &Vec<Rc<RefCell<Card>>>) {
    let castable = find_castable(game_objects);

    let pattern_of_rebirth = castable.iter().find(|(card, _)| card.borrow().is_pattern);
    let is_creature_on_battlefield = game_objects.iter().any(|card| {
        let card = card.borrow();
        card.zone == Zone::Battlefield && card.card_type == CardType::Creature
    });
    let is_pattern_on_battlefield = game_objects.iter().any(|card| {
        let card = card.borrow();
        card.zone == Zone::Battlefield && card.is_pattern
    });
    if let Some((card_ref, payment)) = pattern_of_rebirth {
        if payment.is_some() && is_creature_on_battlefield && !is_pattern_on_battlefield {
            // Target non-sacrifice outlets over sac outlets
            let non_sac_creature = game_objects.iter().find(|card| {
                let card = card.borrow();
                card.zone == Zone::Battlefield
                    && card.card_type == CardType::Creature
                    && !card.is_sac_outlet
            });

            let target = if let Some(creature) = non_sac_creature {
                Rc::clone(creature)
            } else {
                // Otherwise just cast in on a sac outlet
                let sac_creature = game_objects.iter().find(|card| {
                    let card = card.borrow();
                    card.zone == Zone::Battlefield
                        && card.card_type == CardType::Creature
                        && card.is_sac_outlet
                });

                Rc::clone(sac_creature.unwrap())
            };

            cast_spell(card_ref, payment.as_ref().unwrap(), Some(target));
        }
    }
}

pub fn find_castable(
    game_objects: &Vec<Rc<RefCell<Card>>>,
) -> Vec<(Rc<RefCell<Card>>, Option<(Vec<Rc<RefCell<Card>>>, usize)>)> {
    let nonlands_in_hand = game_objects.iter().filter(|card| {
        let card = card.borrow();
        card.zone == Zone::Hand && card.card_type != CardType::Land
    });
    let mut mana_sources: Vec<_> = game_objects
        .iter()
        .filter(|card| {
            let card = card.borrow();
            card.zone == Zone::Battlefield
                && !card.produced_mana.is_empty()
                && !card.is_summoning_sick
                && !card.is_tapped
        })
        .map(Rc::clone)
        .collect();
    mana_sources.sort_by(|a, b| {
        a.borrow()
            .produced_mana
            .len()
            .partial_cmp(&b.borrow().produced_mana.len())
            .unwrap()
    });
    let castable = nonlands_in_hand
        .map(|card| {
            (
                card.clone(),
                find_payment_for(&card.borrow(), &mana_sources),
            )
        })
        .filter(|(_, payment)| payment.is_some());

    castable.collect()
}

pub fn play_land(game_objects: &Vec<Rc<RefCell<Card>>>) {
    let mut lands_in_hand = game_objects
        .iter()
        .filter(|card| {
            let card = card.borrow();
            card.zone == Zone::Hand && card.card_type == CardType::Land
        })
        .map(|card| card)
        .collect::<Vec<_>>();
    lands_in_hand.sort_by(|a, b| {
        a.borrow()
            .produced_mana
            .len()
            .partial_cmp(&b.borrow().produced_mana.len())
            .unwrap()
    });
    // Play the one that produces most colors
    // TODO: Play the one that produces most cards that could be played
    if let Some(land) = lands_in_hand.pop() {
        let mut card = land.borrow_mut();    
    
        println!("Playing land: {}", card.name);
        card.zone = Zone::Battlefield;
    }
}

pub fn draw(
    turn: usize,
    is_first_player: bool,
    deck: &mut Deck,
    game_objects: &mut Vec<Rc<RefCell<Card>>>,
) {
    if turn > 0 || !is_first_player {
        if let Some(mut card) = deck.draw() {
            card.zone = Zone::Hand;
            game_objects.push(Rc::new(RefCell::new(card)))
        }
    }
}

pub fn untap(game_objects: &[Rc<RefCell<Card>>]) {
    for card in game_objects.iter() {
        let mut card = card.borrow_mut();
        card.is_summoning_sick = false;
        card.is_tapped = false;
    }
}

pub fn print_game_state(game_objects: &[Rc<RefCell<Card>>], deck: &Deck, turn: usize) {
    let hand_str = game_objects
        .iter()
        .filter(|card| card.borrow().zone == Zone::Hand)
        .map(|card| card.borrow().name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let battlefield_str = game_objects
        .iter()
        .filter(|card| card.borrow().zone == Zone::Battlefield)
        .map(|card| card.borrow().name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    println!(
        "[Turn {turn:002}][Library]: {} cards remaining.",
        deck.len()
    );
    println!("[Turn {turn:002}][Hand]: {hand_str}");
    println!("[Turn {turn:002}][Battlefield]: {battlefield_str}");
}

pub fn cast_spell(
    card_ref: &Rc<RefCell<Card>>,
    (payment, _floating): &(Vec<Rc<RefCell<Card>>>, usize),
    attach_to: Option<Rc<RefCell<Card>>>,
) {
    let mut card = card_ref.borrow_mut();

    let card_name = &card.name;
    let mana_sources = payment
        .iter()
        .map(|source| source.borrow().name.clone()).collect::<Vec<_>>()
        .join(", ");

    println!("Casting spell {card_name} with: {mana_sources}");

    card.attached_to = attach_to;
    card.zone = Zone::Battlefield;

    if card.card_type == CardType::Creature {
        card.is_summoning_sick = true;
    }

    for mana_source in payment {
        mana_source.borrow_mut().is_tapped = true;
    }
}

pub fn cleanup(game_objects: &Vec<Rc<RefCell<Card>>>) {
    let cards_in_hand = game_objects
        .iter()
        .filter(|card| card.borrow().zone == Zone::Hand)
        .enumerate();

    for (index, card) in cards_in_hand {
        if index >= 7 {
            // TODO: Select the cards to discard by priority
            card.borrow_mut().zone = Zone::Graveyard;
        }
    }
}
