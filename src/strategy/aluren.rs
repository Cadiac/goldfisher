use log::debug;
use std::collections::HashMap;
use std::rc::Rc;

use crate::card::{CardRef, CardType, Zone, SearchFilter};
use crate::deck::Decklist;
use crate::game::GameState;
use crate::mana::{Mana, PaymentAndFloating};
use crate::strategy::Strategy;
use crate::utils::*;

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
    fn cast_mana_dork(&self, game: &mut GameState) -> bool {
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
        game: &mut GameState,
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

    fn combo_status(
        &self,
        game: &GameState,
        include_hand: bool,
        include_battlefield: bool,
    ) -> ComboStatus {
        let game_objects = game.game_objects.iter().filter(|card| {
            (include_hand && is_hand(card)) || (include_battlefield && is_battlefield(card))
        });

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

        let lands = game_objects.clone().filter(is_land).count();

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
    fn decklist() -> Decklist {
        Decklist {
            maindeck: vec![
                ("Birds of Paradise", 4),
                ("Cabal Therapy", 4),
                ("Soul Warden", 1),
                ("Unearth", 2),
                ("Cavern Harpy", 3),
                ("Cloud of Faeries", 1),
                ("Impulse", 4),
                ("Living Wish", 4),
                ("Ray of Revelation", 1),
                ("Wall of Roots", 2),
                ("Intuition", 4),
                ("Raven Familiar", 3),
                ("Wirewood Savage", 1),
                ("Aluren", 4),
                ("City of Brass", 4),
                ("Gemstone Mine", 3),
                ("Hickory Woodlot", 4),
                ("Llanowar Wastes", 2),
                ("Underground River", 2),
                ("Yavimaya Coast", 3),
                ("Forest", 2),
                ("Swamp", 1),
                ("Island", 1),
            ],
            sideboard: vec![
                ("Cavern Harpy", 1),
                ("Wirewood Savage", 1),
                ("Soul Warden", 1),
                ("Maggot Carrier", 1),
                ("Raven Familiar", 1),
                ("Auramancer", 1),
                ("Monk Realist", 1),
                ("Plague Spitter", 1),
                ("Naturalize", 2),
                ("Crippling Fatigue", 1),
                ("Uktabi Orangutan", 1),
                ("Bone Shredder", 1),
                ("Hydroblast", 2),
            ],
        }
    }

    fn is_win_condition_met(&self, game: &GameState) -> bool {
        let hand = self.combo_status(game, true, false);
        let battlefield = self.combo_status(game, false, true);

        // Aluren on the battlefield + Maggot Carrier + Cavern Harpy on the battlefield
        // + Soul warden on the battlefield OR life total is at least 40
        if battlefield.alurens >= 1
            && hand.maggot_carriers >= 1
            && hand.cavern_harpies >= 1
            && (battlefield.soul_wardens >= 1 || game.life_total >= 40)
        {
            return true;
        }

        false
    }

    fn is_keepable_hand(&self, game: &GameState, mulligan_count: usize) -> bool {
        if mulligan_count >= 3 {
            // Just keep the hand with 4 cards
            return true;
        }

        let status = self.combo_status(game, true, false);

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

    fn best_card_to_draw(&self, game: &GameState, search_filter: Option<SearchFilter>) -> &str {
        let status = self.combo_status(game, true, true);

        match search_filter {
            Some(SearchFilter::LivingWish) => {
                if status.alurens >= 1 && status.cavern_harpies >= 1 {
                    if status.wirewood_savages == 0 && status.raven_familiars == 0 {
                        return "Wirewood Savage";
                    } else if status.soul_wardens == 0 {
                        return "Soul Warden";
                    } else {
                        return "Maggot Carrier";
                    }
                }

                if status.cavern_harpies == 0 {
                    return "Cavern Harpy";
                }

                if status.alurens >= 1 && status.wirewood_savages == 0 {
                    return "Wirewood Savage";
                }

                if status.raven_familiars == 0 {
                    return "Raven Familiar";
                }

                if status.soul_wardens == 0 {
                    return "Soul Warden";
                }

                if status.mana_sources < 4 {
                    if status.alurens == 0 {
                        return "Birds of Paradise";
                    } else {
                        // TODO: ignore summoning sickness
                        return "Wall of Roots";
                    }
                }

                "Cavern Harpy"
            }
            None => {
                // TODO: Some actual logic for Intuition
                if status.alurens == 0 {
                    return "Aluren";
                }

                if status.cavern_harpies == 0 {
                    return "Cavern Harpy";
                }

                if status.wirewood_savages == 0 {
                    return "Wirewood Savage";
                }

                if status.raven_familiars == 0 {
                    return "Raven Familiar";
                }

                if status.soul_wardens == 0 {
                    return "Soul Warden";
                }

                if status.mana_sources < 4 {
                    return "City of Brass";
                }

                "Living Wish"
            }
            _ => unimplemented!(),
        }
    }

    fn worst_cards_in_hand(&self, game: &GameState, hand_size: usize) -> Vec<CardRef> {
        let mut ordered_hand = Vec::new();

        let mut lands = Vec::with_capacity(7);
        let mut alurens = Vec::with_capacity(7);
        let mut cavern_harpies = Vec::with_capacity(7);
        let mut draw_engines = Vec::with_capacity(7);
        let mut tutors = Vec::with_capacity(7);
        let mut wincons = Vec::with_capacity(7);
        let mut mana_dorks = Vec::with_capacity(7);

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
            } else if c.name == "Maggot Carrire" || c.name == "Soul Warden" {
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
        for _ in 0..2 {
            if let Some(card) = lands_iter.next() {
                ordered_hand.push(card);
            }
        }

        let mut alurens_iter = alurens.iter();
        for _ in 0..1 {
            if let Some(card) = alurens_iter.next() {
                ordered_hand.push(card);
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
        for _ in 0..1 {
            if let Some(card) = draw_engines_iter.next() {
                ordered_hand.push(card);
            }
        }

        let mut tutors_iter = tutors.iter();
        for _ in 0..1 {
            if let Some(card) = tutors_iter.next() {
                ordered_hand.push(card);
            }
        }

        // Take all mana dorks over extra lands for quick kills
        for card in mana_dorks.iter() {
            ordered_hand.push(card);
        }

        // Then take the rest of the cards, still in priority order
        for card in lands_iter {
            ordered_hand.push(card);
        }
        for card in tutors_iter {
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

    fn take_game_action(&self, game: &mut GameState) -> bool {
        if self.play_land(game) {
            return true;
        }

        let is_aluren_on_battlefield = game
            .game_objects
            .iter()
            .any(|card| is_battlefield(&card) && card.borrow().name == "Aluren");

        if !is_aluren_on_battlefield {
            let castable = game.find_castable();

            let priority_order = [
                "Aluren",
                "Intuition",
                "Living Wish",
                "Impulse",
                "Soul Warden",
                "Maggot Carrier",
                "Cloud of Faeries",
                "Wirewood Savage",
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
                return true;
            }

            let hand = self.combo_status(game, true, false);

            if hand.cloud_of_faeries >= 1 && hand.cavern_harpies >= 1 {
                // Can go for infinite mana, just fake it for now
                game.floating_mana = HashMap::from([
                    (Mana::White, 1000),
                    (Mana::Blue, 1000),
                    (Mana::Black, 1000),
                    (Mana::Red, 1000),
                    (Mana::Green, 1000),
                ]);
                debug!(
                    "[Turn {turn:002}][Action]: Got infinite mana from Cloud of Faeries + Cavern Harpy loop.",
                    turn = game.turn
                );
                game.print_game_state();
            }

            let castable = game.find_castable();

            let priority_order = [
                "Soul Warden",
                "Maggot Carrier",
                "Living Wish",
                "Intuition",
                "Wirewood Savage",
                "Raven Familiar",
                "Wall of Roots", // Wall of Roots produces {G}
            ];

            for card_name in priority_order {
                if self.cast_named(game, castable.clone(), card_name) {
                    return true;
                }
            }

            let battlefield = self.combo_status(game, false, true);
            let something_to_bounce = battlefield.maggot_carriers > 0
                || battlefield.cloud_of_faeries > 0
                || battlefield.raven_familiars > 0
                || battlefield.wirewood_savages > 0;

            if hand.cavern_harpies >= 1 && something_to_bounce {
                if self.cast_named(game, castable.clone(), "Cavern Harpy") {
                    return true;
                }
            }

            if self.cast_mana_dork(game) {
                return true;
            }
        }

        return false;
    }
}
