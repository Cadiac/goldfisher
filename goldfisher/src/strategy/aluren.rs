use log::{debug, warn};
use std::collections::HashMap;
use std::rc::Rc;

use crate::card::{CardRef, CardType, Zone};
use crate::deck::Decklist;
use crate::game::Game;
use crate::mana::PaymentAndFloating;
use crate::strategy::Strategy;
use crate::utils::*;

const DEFAULT_DECKLIST: &str = include_str!("../../resources/aluren.txt");
pub const NAME: &str = "Premodern - Aluren";

struct ComboStatus {
    mana_sources: usize,
    lands: usize,
    alurens: usize,
    cloud_of_faeries: usize,
    cavern_harpies: usize,
    wirewood_savages: usize,
    raven_familiars: usize,
    soul_wardens: usize,
    maggot_carriers: usize,
}

pub struct Aluren {}

impl Aluren {
    fn cast_mana_dork(&self, game: &mut Game) -> bool {
        let castable = game.find_castable();

        let mut mana_dorks = castable
            .iter()
            .filter(|(card, _)| is_mana_dork(&card))
            .collect::<Vec<_>>();

        // Cast the one that produces most colors
        mana_dorks.sort_by(|(a, _), (b, _)| sort_by_best_mana_to_play(a, b));

        if let Some((card_ref, payment)) = mana_dorks.last() {
            game.cast_spell(self, card_ref, payment.as_ref().unwrap(), None);
            return true;
        }

        false
    }

    fn cast_named(
        &self,
        game: &mut Game,
        castable: Vec<(CardRef, Option<PaymentAndFloating>)>,
        card_name: &str,
    ) -> bool {
        if let Some((card_ref, payment)) =
            castable.iter().find(|(c, _)| c.borrow().name == card_name)
        {
            game.cast_spell(self, card_ref, payment.as_ref().unwrap(), None);
            return true;
        }

        false
    }

    fn combo_status(&self, game: &Game, zones: Vec<Zone>) -> ComboStatus {
        let game_objects = game
            .game_objects
            .iter()
            .filter(|card| zones.contains(&card.borrow().zone));

        let alurens = game_objects
            .clone()
            .filter(|card| card.borrow().name == "Aluren")
            .count();
        let cavern_harpies = game_objects
            .clone()
            .filter(|card| card.borrow().name == "Cavern Harpy")
            .count();
        let wirewood_savages = game_objects
            .clone()
            .filter(|card| card.borrow().name == "Wirewood Savage")
            .count();
        let raven_familiars = game_objects
            .clone()
            .filter(|card| card.borrow().name == "Raven Familiar")
            .count();
        let cloud_of_faeries = game_objects
            .clone()
            .filter(|card| card.borrow().name == "Cloud of Faeries")
            .count();

        let soul_wardens = game_objects
            .clone()
            .filter(|card| card.borrow().name == "Soul Warden")
            .count();

        let maggot_carriers = game_objects
            .clone()
            .filter(|card| card.borrow().name == "Maggot Carrier")
            .count();

        let lands = game_objects
            .clone()
            .filter(|card| is_card_type(card, CardType::Land))
            .count();

        let mana_sources = game_objects
            .clone()
            .filter(|card| is_mana_source(card) && !is_single_use_mana(card))
            .count();

        ComboStatus {
            lands,
            mana_sources,
            alurens,
            cavern_harpies,
            cloud_of_faeries,
            wirewood_savages,
            raven_familiars,
            soul_wardens,
            maggot_carriers,
        }
    }
}

impl Strategy for Aluren {
    fn name(&self) -> String { NAME.to_owned() }

    fn default_decklist(&self) -> Decklist {
        DEFAULT_DECKLIST.parse::<Decklist>().unwrap()
    }

    fn is_keepable_hand(&self, game: &Game, mulligan_count: usize) -> bool {
        if mulligan_count >= 3 {
            // Just keep the hand with 4 cards
            return true;
        }

        let status = self.combo_status(game, vec![Zone::Hand]);

        if status.lands == 0 {
            // Always mulligan zero land hands
            return false;
        }

        if status.mana_sources >= 6 {
            // Also mulligan too mana source heavy hands
            return false;
        }

        if status.lands == 1 && status.mana_sources <= 2 {
            // One landers with just max one mana dork get automatically mulliganed too
            return false;
        }

        // Having Aluren with Cavern Harpy or draw engine is always good enough
        if status.alurens >= 1
            && (status.cavern_harpies >= 1
                || (status.raven_familiars >= 1 || status.wirewood_savages >= 1))
        {
            return true;
        }

        // If we have already taken a mulligans this should be good enough
        if status.alurens >= 1 && mulligan_count > 0 {
            return true;
        }

        // TODO: Give some value to tutors and draw spells

        // Otherwise take a mulligan
        false
    }

    fn select_best(
        &self,
        game: &Game,
        cards: HashMap<String, Vec<CardRef>>,
    ) -> Option<CardRef> {
        let status = self.combo_status(game, vec![Zone::Hand, Zone::Battlefield]);
        let battlefield = self.combo_status(game, vec![Zone::Battlefield]);

        if battlefield.alurens >= 1 {
            if status.cavern_harpies == 0 {
                let card = find_named(&cards, "Cavern Harpy");
                if card.is_some() {
                    return card;
                }
            }

            if status.soul_wardens == 0
                && status.wirewood_savages == 0
                && status.raven_familiars == 0
            {
                let card = find_named(&cards, "Wirewood Savage");
                if card.is_some() {
                    return card;
                }
                let card = find_named(&cards, "Raven Familiar");
                if card.is_some() {
                    return card;
                }
            }

            if status.soul_wardens == 0 {
                let card = find_named(&cards, "Soul Warden");
                if card.is_some() {
                    return card;
                }
            }

            if status.maggot_carriers == 0 {
                let card = find_named(&cards, "Maggot Carrier");
                if card.is_some() {
                    return card;
                }
            }

            if status.wirewood_savages == 0 && status.raven_familiars == 0 {
                let card = find_named(&cards, "Wirewood Savage");
                if card.is_some() {
                    return card;
                }
                let card = find_named(&cards, "Raven Familiar");
                if card.is_some() {
                    return card;
                }
            }

            let card = find_named(&cards, "Cloud of Faeries");
            if card.is_some() {
                return card;
            }
        }

        if battlefield.alurens == 0 {
            if status.alurens == 0 {
                let card = find_named(&cards, "Aluren");
                if card.is_some() {
                    return card;
                }
            }
            if status.mana_sources < 4 {
                if game.available_land_drops > 0 {
                    let mut lands: Vec<CardRef> = cards
                        .values()
                        .flatten()
                        .filter(|card| is_card_type(card, CardType::Land))
                        .cloned()
                        .collect();
                    lands.sort_by(sort_by_best_mana_to_play);

                    if let Some(best_land) = lands.last() {
                        let name = &best_land.borrow().name;
                        let card = cards
                            .get(name.as_str())
                            .and_then(|copies| copies.first())
                            .cloned();
                        if card.is_some() {
                            return card;
                        }
                    }
                }
                let card = find_named(&cards, "Birds of Paradise");
                if card.is_some() {
                    return card;
                }
                let card = find_named(&cards, "Wall of Roots");
                if card.is_some() {
                    return card;
                }
            }
            if status.cavern_harpies == 0 {
                let card = find_named(&cards, "Cavern Harpy");
                if card.is_some() {
                    return card;
                }
            }
            if status.wirewood_savages == 0 && status.raven_familiars == 0 {
                let card = find_named(&cards, "Raven Familiar");
                if card.is_some() {
                    return card;
                }
                let card = find_named(&cards, "Wirewood Savage");
                if card.is_some() {
                    return card;
                }
            }
            if status.soul_wardens == 0 {
                let card = find_named(&cards, "Soul Warden");
                if card.is_some() {
                    return card;
                }
            }
            if status.maggot_carriers == 0 {
                let card = find_named(&cards, "Maggot Carrier");
                if card.is_some() {
                    return card;
                }
            }
        }

        let card = find_named(&cards, "Living Wish");
        if card.is_some() {
            return card;
        }

        let card = find_named(&cards, "Intuition");
        if card.is_some() {
            return card;
        }

        let card = find_named(&cards, "Impulse");
        if card.is_some() {
            return card;
        }

        // Otherwise just pick anything
        cards.values().flatten().cloned().next()
    }

    fn select_intuition(&self, game: &Game) -> Vec<CardRef> {
        let searchable = apply_search_filter(game, &None);
        if let Some(found) = self.select_best(game, group_by_name(searchable)) {
            let mut cards = Vec::with_capacity(3);

            let priority_list = if found.borrow().name == "Aluren" {
                // Just grab three copies of Aluren, assume there are enough in the deck
                vec!["Aluren"]
            } else if found.borrow().name == "Cavern Harpy" {
                vec!["Cavern Harpy", "Unearth"]
            } else if found.borrow().name == "Wirewood Savage"
                || found.borrow().name == "Raven Familiar"
            {
                vec!["Wirewood Savage", "Raven Familiar"]
            } else if found.borrow().name == "Birds of Paradise" {
                vec![
                    "Birds of Paradise",
                    "Wall of Roots",
                    "City of Brass",
                    "Gemstone Mine",
                ]
            } else if found.borrow().card_type == CardType::Land {
                vec![
                    "City of Brass",
                    "Gemstone Mine",
                    "Llawnowar Wastes",
                    "Forest",
                ]
            } else if found.borrow().card_type == CardType::Creature {
                cards.push(found);
                vec![
                    "Unearth",
                    "Raven Familiar",
                    "Soul Warden",
                    "Wirewood Savage",
                ]
            } else {
                // Can't Intuition for Living Wish or the game might become unwinnable.
                // Intuition as target also doesn't make any sense.
                // Shouldn't really even cast Intuition at this point - but grab some random stuff instead.
                vec![
                    "Unearth",
                    "Raven Familiar",
                    "Soul Warden",
                    "Wirewood Savage",
                    "Impulse",
                ]
            };

            for card in find_n_with_priority(game, 3 - cards.len(), &priority_list) {
                cards.push(card);
            }

            debug!("Found {:?}", cards);
            if cards.len() != 3 && cards.len() != game.deck.len() {
                warn!("Unexpected number of cards found, ignoring...")
            }

            cards
        } else {
            // Empty deck
            vec![]
        }
    }

    fn discard_to_hand_size(&self, game: &Game, hand_size: usize) -> Vec<CardRef> {
        let mut ordered_hand = Vec::new();

        let mut lands = Vec::with_capacity(7);
        let mut alurens = Vec::with_capacity(7);
        let mut cavern_harpies = Vec::with_capacity(7);
        let mut draw_engines = Vec::with_capacity(7);
        let mut tutors = Vec::with_capacity(7);
        let mut wincons = Vec::with_capacity(7);
        let mut mana_dorks = Vec::with_capacity(7);

        let is_aluren_on_battlefield = game
            .game_objects
            .iter()
            .any(|card| is_battlefield(&card) && card.borrow().name == "Aluren");

        let mut other_cards = Vec::with_capacity(7);

        let hand = game.game_objects.iter().filter(is_hand);

        for card in hand {
            let c = card.borrow();

            if c.card_type == CardType::Land {
                lands.push(card.clone());
            } else if c.name == "Aluren" {
                alurens.push(card.clone());
            } else if c.name == "Cavern Harpy" {
                cavern_harpies.push(card.clone());
            } else if c.name == "Wirewood Savage" || c.name == "Raven Familiar" {
                draw_engines.push(card.clone());
            } else if c.name == "Living Wish" || c.name == "Intuition" {
                tutors.push(card.clone());
            } else if c.name == "Maggot Carrier" || c.name == "Soul Warden" {
                wincons.push(card.clone());
            } else if is_aluren_on_battlefield && c.name == "Unearth" {
                wincons.push(card.clone());
            } else if c.card_type == CardType::Creature && !c.produced_mana.is_empty() {
                mana_dorks.push(card.clone());
            } else {
                other_cards.push(card.clone());
            }
        }

        lands.sort_by(sort_by_best_mana_to_play);

        // First keep a balanced mix of lands and combo pieces
        // Prefer lands that produce the most colors of mana (sorted to the end of the iter)
        let mut lands_iter = lands.iter().rev();
        if !is_aluren_on_battlefield {
            for _ in 0..2 {
                if let Some(card) = lands_iter.next() {
                    ordered_hand.push(card);
                }
            }
        }

        let mut alurens_iter = alurens.iter();
        if !is_aluren_on_battlefield {
            for _ in 0..1 {
                if let Some(card) = alurens_iter.next() {
                    ordered_hand.push(card);
                }
            }
        }

        // Try to keep the wincons in hand
        for card in wincons.iter() {
            ordered_hand.push(card);
        }

        let mut cavern_harpies_iter = cavern_harpies.iter();
        for _ in 0..1 {
            if let Some(card) = cavern_harpies_iter.next() {
                ordered_hand.push(card);
            }
        }

        let mut draw_engines_iter = draw_engines.iter();
        if !is_aluren_on_battlefield {
            for _ in 0..1 {
                if let Some(card) = draw_engines_iter.next() {
                    ordered_hand.push(card);
                }
            }
        }

        // Take all tutors
        for card in tutors.iter() {
            ordered_hand.push(card);
        }

        // Take all mana dorks over extra lands for quick kills
        for card in mana_dorks.iter() {
            ordered_hand.push(card);
        }

        // Then take the rest of the cards, still in priority order
        for card in lands_iter {
            ordered_hand.push(card);
        }
        for card in draw_engines_iter {
            ordered_hand.push(card);
        }
        for card in other_cards.iter() {
            ordered_hand.push(card);
        }
        for card in cavern_harpies_iter {
            ordered_hand.push(card);
        }
        for card in alurens_iter {
            ordered_hand.push(card);
        }

        ordered_hand
            .iter()
            .skip(hand_size)
            .map(|card| Rc::clone(card))
            .collect()
    }

    fn take_game_action(&self, game: &mut Game) -> bool {
        if self.play_land(game) {
            return true;
        }

        let battlefield = self.combo_status(game, vec![Zone::Battlefield]);
        let hand = self.combo_status(game, vec![Zone::Hand]);

        if battlefield.alurens == 0 {
            let castable = game.find_castable();

            if hand.alurens == 0 {
                let priority_order = [
                    "Aluren",
                    "Intuition",
                    "Living Wish",
                    "Impulse",
                    "Soul Warden",
                    "Maggot Carrier",
                    "Cloud of Faeries",
                    "Raven Familiar",
                    "Wirewood Savage",
                    "Cavern Harpy",
                ];

                for card_name in priority_order {
                    if self.cast_named(game, castable.clone(), card_name) {
                        return true;
                    }
                }

                if self.cast_mana_dork(game) {
                    return true;
                }
            } else {
                if self.cast_named(game, castable.clone(), "Aluren") {
                    return true;
                }

                if self.cast_mana_dork(game) {
                    return true;
                }

                let priority_order = [
                    "Intuition",
                    "Living Wish",
                    "Impulse",
                    "Soul Warden",
                    "Maggot Carrier",
                    "Cloud of Faeries",
                    "Raven Familiar",
                    "Wirewood Savage",
                    "Cavern Harpy",
                ];

                for card_name in priority_order {
                    if self.cast_named(game, castable.clone(), card_name) {
                        return true;
                    }
                }
            }
        } else {
            let cavern_harpy_on_battlefield = game
                .game_objects
                .iter()
                .find(|card| is_battlefield(card) && card.borrow().name == "Cavern Harpy");

            if let Some(card) = cavern_harpy_on_battlefield {
                // Return any Cavern Harpies sitting on the battlefield back to hand
                debug!(
                    "[Turn {turn:002}][Action]: Returning \"Cavern Harpy\" back to hand.",
                    turn = game.turn
                );
                card.borrow_mut().zone = Zone::Hand;
                game.take_damage(1);
                return true;
            }

            let mut castable = game.find_castable();

            // Cast any mana dorks for free
            if self.cast_mana_dork(game) {
                return true;
            }

            let have_cavern_harpy = game.game_objects.iter().any(|card| {
                let card = card.borrow();
                (card.zone == Zone::Battlefield || card.zone == Zone::Hand)
                    && card.name == "Cavern Harpy"
            });
            let have_soul_warden = game.game_objects.iter().any(|card| {
                let card = card.borrow();
                (card.zone == Zone::Battlefield || card.zone == Zone::Hand)
                    && card.name == "Soul Warden"
            });
            let have_draw_engine = game.game_objects.iter().any(|card| {
                let card = card.borrow();
                (card.zone == Zone::Battlefield || card.zone == Zone::Hand)
                    && (card.name == "Wirewood Savage" || card.name == "Raven Familiar")
            });

            let mut priority_order = vec![
                "Soul Warden",
                "Maggot Carrier",
                "Wirewood Savage",
                "Living Wish",
            ];

            if !(have_cavern_harpy && have_soul_warden && have_draw_engine) {
                priority_order.push("Intuition");
            }
            for card_name in priority_order {
                if self.cast_named(game, castable.clone(), card_name) {
                    return true;
                }
            }

            // If there's still deck left to cast Raven Familiars and still pass the turn
            if game.deck.len() > 1 && self.cast_named(game, castable.clone(), "Raven Familiar") {
                return true;
            }

            let land_count = game
                .game_objects
                .iter()
                .filter(|card| is_card_type(card, CardType::Land) && is_battlefield(card))
                .count();

            if hand.cloud_of_faeries >= 1
                && hand.cavern_harpies >= 1
                && land_count > 0
                && game.floating_mana.values().sum::<usize>() < 5
            {
                // Can generate mana at the cost of life, or infinite if we also have soul warden
                game.float_mana();
                // Need to refresh this so that no floating mana is lost
                castable = game.find_castable();

                if self.cast_named(game, castable.clone(), "Cloud of Faeries") {
                    return true;
                }
            }

            // Maybe some combo pieces have been discarded
            let graveyard = self.combo_status(game, vec![Zone::Graveyard]);
            if graveyard.maggot_carriers >= 1
                || graveyard.soul_wardens >= 1
                || graveyard.wirewood_savages >= 1
                || graveyard.raven_familiars >= 1
                || graveyard.cloud_of_faeries >= 1
                || graveyard.cavern_harpies >= 1
            {
                if self.cast_named(game, castable.clone(), "Unearth") {
                    return true;
                }
            }

            if game.deck.len() <= 1 && hand.maggot_carriers == 0 && battlefield.maggot_carriers == 0
            {
                // Have to pass the turn, probably due to lack of mana :(
                return false;
            }

            let something_to_bounce = battlefield.maggot_carriers > 0
                || battlefield.cloud_of_faeries > 0
                || battlefield.raven_familiars > 0;

            if hand.cavern_harpies >= 1 && (something_to_bounce || battlefield.wirewood_savages > 0)
            {
                if self.cast_named(game, castable.clone(), "Cavern Harpy") {
                    return true;
                }
            }
        }

        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_game(cards_and_zones: Vec<(&str, Zone)>, strategy: &impl Strategy) -> Game {
        let game = Game::new(&strategy.default_decklist()).unwrap();

        for (name, zone) in cards_and_zones {
            game.game_objects
                .iter()
                .find(|card| card.borrow().name == name && card.borrow().zone != zone)
                .map(|card| card.borrow_mut().zone = zone);
        }

        game
    }

    fn assert_best_card(expected: &str, cards_and_zones: Vec<(&str, Zone)>) {
        let strategy = Aluren {};
        let game = setup_game(cards_and_zones, &strategy);
        let cards = group_by_name(
            game.game_objects
                .iter()
                .filter(|card| card.borrow().zone == Zone::Library)
                .cloned()
                .collect(),
        );

        let best_card = Aluren {}.select_best(&game, cards);

        assert_eq!(true, best_card.is_some());
        assert_eq!(expected, best_card.unwrap().borrow().name);
    }

    fn assert_best_card_from_sideboard(expected: &str, cards_and_zones: Vec<(&str, Zone)>) {
        let strategy = Aluren {};
        let game = setup_game(cards_and_zones, &strategy);
        let cards = group_by_name(
            game.deck
                .sideboard
                .iter()
                .filter(|card| {
                    card.borrow().card_type == CardType::Creature
                        || card.borrow().card_type == CardType::Land
                })
                .cloned()
                .collect(),
        );

        let best_card = Aluren {}.select_best(&game, cards);

        assert_eq!(true, best_card.is_some());
        assert_eq!(expected, best_card.unwrap().borrow().name);
    }

    #[test]
    fn it_finds_correct_best_cards_without_aluren() {
        assert_best_card("Aluren", vec![]);
        assert_best_card("City of Brass", vec![("Aluren", Zone::Hand)]);
        assert_best_card(
            "Cavern Harpy",
            vec![
                ("Aluren", Zone::Hand),
                ("City of Brass", Zone::Hand),
                ("City of Brass", Zone::Hand),
                ("City of Brass", Zone::Hand),
                ("City of Brass", Zone::Hand),
            ],
        );
        assert_best_card(
            "Raven Familiar",
            vec![
                ("Aluren", Zone::Hand),
                ("Cavern Harpy", Zone::Hand),
                ("City of Brass", Zone::Hand),
                ("City of Brass", Zone::Hand),
                ("City of Brass", Zone::Hand),
                ("City of Brass", Zone::Hand),
            ],
        );
    }

    #[test]
    fn it_finds_correct_best_cards_with_aluren() {
        assert_best_card("Cavern Harpy", vec![("Aluren", Zone::Battlefield)]);
        assert_best_card(
            "Wirewood Savage",
            vec![("Aluren", Zone::Battlefield), ("Cavern Harpy", Zone::Hand)],
        );
        assert_best_card(
            "Wirewood Savage",
            vec![
                ("Aluren", Zone::Battlefield),
                ("Cavern Harpy", Zone::Hand),
                ("Soul Warden", Zone::Hand),
            ],
        );
        assert_best_card(
            "Raven Familiar",
            vec![
                ("Aluren", Zone::Battlefield),
                ("Cavern Harpy", Zone::Hand),
                ("Wirewood Savage", Zone::Graveyard),
            ],
        );
        assert_best_card(
            "Soul Warden",
            vec![
                ("Aluren", Zone::Battlefield),
                ("Cavern Harpy", Zone::Hand),
                ("Raven Familiar", Zone::Battlefield),
            ],
        );
        assert_best_card(
            "Soul Warden",
            vec![
                ("Aluren", Zone::Battlefield),
                ("Cavern Harpy", Zone::Hand),
                ("Wirewood Savage", Zone::Battlefield),
            ],
        );
        // Maggot carrier is found from sideboard
        assert_best_card_from_sideboard(
            "Maggot Carrier",
            vec![
                ("Aluren", Zone::Battlefield),
                ("Cavern Harpy", Zone::Hand),
                ("Raven Familiar", Zone::Battlefield),
                ("Soul Warden", Zone::Battlefield),
            ],
        )
    }
}
