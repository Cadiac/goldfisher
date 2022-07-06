use std::rc::Rc;
use std::cell::{RefCell};

use goldfisher::card::{Card, CardType};
use goldfisher::deck::create_deck;
use goldfisher::mana::can_pay_for;

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
    let mut battlefield: Vec<Rc<RefCell<Card>>> = Vec::new();

    let is_first_player = true;

    // Take the opening 7
    for _ in 0..7 {
        if let Some(card) = deck.draw() {
            hand.push(card)
        }
    }

    loop {
        for card in battlefield.iter_mut() {
            card.borrow_mut().is_summoning_sick = false;
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
        // TODO: Play the one that produces most cards that could be played
        if let Some(land) = lands_in_hand.pop() {
            battlefield.push(Rc::new(RefCell::new(land)));
        }


        // 2. Figure out which cards in our hand we can pay for
        let nonlands_in_hand = hand
            .iter_mut()
            .filter(|card| card.card_type != CardType::Land);

        let mana_sources = battlefield.iter().filter(|card| {
            let card = card.borrow();
            !card.produced_mana.is_empty() && !card.is_summoning_sick && !card.is_tapped
        });

        let mut available_mana: Vec<_> = mana_sources
            .map(|card| card.borrow().produced_mana.clone())
            .collect();
        available_mana.sort_by(|a, b| a.len().partial_cmp(&b.len()).unwrap());

        let mut castable = nonlands_in_hand
            .filter(|card| can_pay_for(card, &available_mana));


        // 3. If we have a Pattern of Rebirth in hand cast it on a creature if we don't have one yet
        let pattern_of_rebirth = castable.find(|card| card.is_pattern);
        let is_creature_on_battlefield = battlefield
            .iter()
            .any(|card| card.borrow().card_type == CardType::Creature);
        let is_pattern_on_battlefield = !battlefield.iter().any(|card| card.borrow().is_pattern);

        if pattern_of_rebirth.is_some() && is_creature_on_battlefield && !is_pattern_on_battlefield {
            // Target non-sacrifice outlets over sac outlets

            let non_sac_creature = battlefield
                .iter()
                .find(|card| card.borrow().card_type == CardType::Creature && !card.borrow().is_sac_outlet);

            if let Some(creature) = non_sac_creature {
                pattern_of_rebirth.unwrap().attached_to = Some(Rc::clone(creature));
            } else {
                // Otherwise just cast in on a sac outlet
                let sac_creature = battlefield
                    .iter()
                    .find(|card| card.borrow().card_type == CardType::Creature && card.borrow().is_sac_outlet);

                if let Some(creature) = sac_creature {
                    pattern_of_rebirth.unwrap().attached_to = Some(Rc::clone(creature));
                }
            }
        }

        // 4. If we have Pattern of Rebirth already cast sac outlets
        if is_pattern_on_battlefield {
            
        }

        // N. Do we have it?

        // If not, take another turn

        turn += 1;
    }
}
