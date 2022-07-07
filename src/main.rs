use std::cell::RefCell;
use std::rc::Rc;

use goldfisher::card::{Card, CardType, Zone};
use goldfisher::deck::{Deck, create_deck};
use goldfisher::mana::find_payment_for;

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

    let mut turn = 1;
    let mut game_objects: Vec<Rc<RefCell<Card>>> = Vec::new();

    let is_first_player = true;

    // Take the opening 7
    for _ in 0..7 {
        if let Some(mut card) = deck.draw() {
            card.zone = Zone::Hand;
            game_objects.push(Rc::new(RefCell::new(card)))
        }
    }

    loop {
        for card in game_objects.iter_mut() {
            card.borrow_mut().is_summoning_sick = false;
        }

        // 0. Draw a card for the turn
        if turn > 0 || !is_first_player {
            if let Some(mut card) = deck.draw() {
                card.zone = Zone::Hand;
                game_objects.push(Rc::new(RefCell::new(card)))
            }
        }

        print_game_state(&game_objects, &deck, turn);

        // 1. Play a land card, preferring lands with most colors produced
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
            land.borrow_mut().zone = Zone::Battlefield;
        }

        // 2. Figure out which cards in our hand we can pay for
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

        let mut castable = nonlands_in_hand
            .map(|card| {
                (
                    card.clone(),
                    find_payment_for(&card.borrow(), &mana_sources),
                )
            })
            .filter(|(_, payment)| payment.is_some());

        // 3. If we have a Pattern of Rebirth in hand cast it on a creature if we don't have one yet
        let pattern_of_rebirth = castable.find(|(card, _)| card.borrow().is_pattern);

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

                cast_card(card_ref, &payment.unwrap(), Some(target));
            }
        }

        // 4. If we have Pattern of Rebirth already cast any sac outlets
        let sac_creature = castable.find(|(card, _)| {
            let card = card.borrow();
            card.card_type == CardType::Creature && card.is_sac_outlet
        });

        if let Some((card_ref, payment)) = sac_creature {
            cast_card(card_ref, &payment.unwrap(), None);
        }

        // 5. Otherwise cast any creatures
        let creature = castable.find(|(card, _)| {
            card.borrow().card_type == CardType::Creature
        });

        if let Some((card_ref, payment)) = creature {
            cast_card(card_ref, &payment.unwrap(), None);
        }

        // N. Do we have it?
        let have_sac_outlet = game_objects.iter().any(|card| {
            let card = card.borrow();
            card.zone == Zone::Battlefield && card.is_sac_outlet
        });
        let have_pattern_attached = game_objects.iter().any(|card| {
            let card = card.borrow();
            card.zone == Zone::Battlefield && card.is_pattern
        });
        // TODO: test if its attached to the only sac outlet
        if have_sac_outlet && have_pattern_attached {
            println!("Won the game on turn {turn}!");
            print_game_state(&game_objects, &deck, turn);
            return;
        }

        // If not, take another turn

        turn += 1;
    }
}

fn print_game_state(game_objects: &[Rc<RefCell<Card>>], deck: &Deck, turn: usize) {
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

fn cast_card(card_ref: Rc<RefCell<Card>>, payment: &[Rc<RefCell<Card>>], attach_to: Option<Rc<RefCell<Card>>>) {
    let mut card = card_ref.borrow_mut();
    card.attached_to = attach_to;
    card.zone = Zone::Battlefield;
    for mana_source in payment {
        mana_source.borrow_mut().is_tapped = true;
    }
}