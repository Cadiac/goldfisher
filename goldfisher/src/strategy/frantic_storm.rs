use log::debug;
use std::collections::HashMap;
use std::rc::Rc;

use crate::card::{CardRef, CardType, Zone};
use crate::deck::Decklist;
use crate::game::Game;
use crate::mana::PaymentAndFloating;
use crate::strategy::Strategy;
use crate::utils::*;

const DEFAULT_DECKLIST: &str = include_str!("../../resources/frantic-storm.txt");
pub const NAME: &str = "Premodern - Frantic Storm";

struct ComboStatus {
    lands: usize,
    mana_sources: usize,
    cost_reducers: usize,
    cantrips: usize,
    cloud_of_faeries: usize,
}

pub struct FranticStorm {
    is_storming: bool,
}

impl FranticStorm {
    pub fn new() -> Self {
        Self { is_storming: false }
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

        let cost_reducers = game_objects
            .clone()
            .filter(|card| {
                is_named(card, "Helm of Awakening") || is_named(card, "Sapphire Medallion")
            })
            .count();

        let cloud_of_faeries = game_objects
            .clone()
            .filter(|card| is_named(card, "Cloud of Faeries"))
            .count();

        let cantrips = game_objects
            .clone()
            .filter(|card| {
                is_named(card, "Frantic Search")
                    || is_named(card, "Impulse")
                    || is_named(card, "Meditate")
                    || is_named(card, "Sleight of Hand")
                    || is_named(card, "Merchant Scroll")
                    || is_named(card, "Words of Wisdom")
            })
            .count();

        let lands = game_objects
            .clone()
            .filter(|card| is_card_type(card, CardType::Land))
            .count();

        let mana_sources = game_objects
            .clone()
            .filter(|card| is_card_type(card, CardType::Land) || is_mana_source(card))
            .count();

        ComboStatus {
            lands,
            cloud_of_faeries,
            cantrips,
            cost_reducers,
            mana_sources,
        }
    }
}

impl Strategy for FranticStorm {
    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn default_decklist(&self) -> Decklist {
        DEFAULT_DECKLIST.parse::<Decklist>().unwrap()
    }

    fn cleanup(&mut self) {
        self.is_storming = false;
    }

    fn is_keepable_hand(&self, game: &Game, mulligan_count: usize) -> bool {
        if mulligan_count >= 3 {
            // Just keep any hand with 4 cards
            return true;
        }

        let hand = self.combo_status(game, vec![Zone::Hand]);

        // The "perfect" hand
        if hand.cost_reducers >= 1 && hand.mana_sources >= 2 && hand.cantrips >= 1 {
            return true;
        }

        if hand.lands == 0 {
            // Always mulligan zero or one land hands
            return false;
        }

        if hand.mana_sources >= 6 {
            // Also mulligan too mana source heavy hands
            return false;
        }

        return true;
    }

    fn select_best(&self, game: &Game, cards: HashMap<String, Vec<CardRef>>) -> Option<CardRef> {
        let status = self.combo_status(game, vec![Zone::Hand, Zone::Battlefield]);

        if status.lands < 2 {
            let card = find_named(&cards, "Island");
            if card.is_some() {
                return card;
            }
        }

        if status.cost_reducers == 0 {
            for name in ["Helm of Awakening", "Sapphire Medallion"] {
                let card = find_named(&cards, name);
                if card.is_some() {
                    return card;
                }
            }
        }

        if game.storm >= 5 {
            if let Some(card) = find_named(&cards, "Brain Freeze") {
                return Some(card);
            }
        }

        for name in [
            "Meditate",
            "Impulse",
            "Cloud of Faeries",
            "Snap",
            "Merchant Scroll",
            "Cunning Wish",
            "Frantic Search",
            "Sleight of hand",
            "Brain Freeze",
            "Turnabout",
            "Words of Wisdom",
            "Lotus Petal",
        ] {
            let card = find_named(&cards, name);
            if card.is_some() {
                return card;
            }
        }

        // Otherwise just pick anything
        cards.values().flatten().cloned().next()
    }

    fn discard_to_hand_size(&self, game: &Game, hand_size: usize) -> Vec<CardRef> {
        let mut ordered_hand = Vec::new();

        let mut lands = Vec::with_capacity(7);
        let mut cost_reducers = Vec::with_capacity(7);
        let mut cantrips = Vec::with_capacity(7);
        let mut tutors = Vec::with_capacity(7);
        let mut untappers = Vec::with_capacity(7);
        let mut wincons = Vec::with_capacity(7);
        let mut petals = Vec::with_capacity(7);
        let mut other_cards = Vec::with_capacity(7);

        let cost_reducers_on_battlefield = game
            .game_objects
            .iter()
            .filter(|card| {
                is_zone(card, &Zone::Battlefield)
                    && (is_named(card, "Helm of Awakening") || is_named(card, "Sapphire Medallion"))
            })
            .count();

        let hand = game.game_objects.iter().filter(is_hand);

        let mut unordered_hand_len: usize = 0;
        for card in hand {
            unordered_hand_len += 1;
            if is_card_type(&card, CardType::Land) {
                lands.push(card.clone());
            } else if is_named(&card, "Helm of Awakening") || is_named(&card, "Sapphire Medallion")
            {
                cost_reducers.push(card.clone());
            } else if is_named(&card, "Frantic Search")
                || is_named(&card, "Meditate")
                || is_named(&card, "Impulse")
                || is_named(&card, "Sleight of Hand")
            {
                cantrips.push(card.clone());
            } else if is_named(&card, "Merchant Scroll")
                || is_named(&card, "Cunning Wish")
                || is_named(&card, "Intuition")
            {
                tutors.push(card.clone());
            } else if is_named(&card, "Brain Freeze") {
                wincons.push(card.clone());
            } else if is_named(&card, "Cloud of Faeries")
                || is_named(&card, "Snap")
                || is_named(&card, "Turnabout")
            {
                untappers.push(card.clone());
            } else if is_named(&card, "Lotus Petal") {
                petals.push(card.clone());
            } else {
                other_cards.push(card.clone());
            }
        }

        lands.sort_by(sort_by_best_mana_to_play);

        // First keep a balanced mix of lands and cost reducers
        let mut lands_iter = lands.iter().rev();
        if cost_reducers_on_battlefield == 0 {
            for _ in 0..2 {
                if let Some(card) = lands_iter.next() {
                    ordered_hand.push(card);
                }
            }
        }

        let mut cost_reducers_iter = cost_reducers.iter();
        if cost_reducers_on_battlefield < 2 {
            for _ in 0..1 {
                if let Some(card) = cost_reducers_iter.next() {
                    ordered_hand.push(card);
                }
            }
        }

        // Try to keep the wincons in hand
        for card in wincons.iter() {
            ordered_hand.push(card);
        }

        let mut untappers_iter = untappers.iter();
        for _ in 0..2 {
            if let Some(card) = untappers_iter.next() {
                ordered_hand.push(card);
            }
        }

        let mut cantrips_iter = cantrips.iter();
        for _ in 0..2 {
            if let Some(card) = cantrips_iter.next() {
                ordered_hand.push(card);
            }
        }

        // Take all tutors
        for card in tutors.iter() {
            ordered_hand.push(card);
        }

        // Take all petals over extra lands for quick kills
        for card in petals.iter() {
            ordered_hand.push(card);
        }

        for card in untappers_iter {
            ordered_hand.push(card);
        }
        // Then take the rest of the cards, still in priority order
        for card in lands_iter {
            ordered_hand.push(card);
        }
        for card in cantrips_iter {
            ordered_hand.push(card);
        }
        for card in cost_reducers_iter {
            ordered_hand.push(card);
        }
        for card in other_cards.iter() {
            ordered_hand.push(card);
        }

        assert!(
            ordered_hand.len() == unordered_hand_len,
            "mismatched ordered and unordered hand len"
        );

        ordered_hand
            .iter()
            .skip(hand_size)
            .map(|card| Rc::clone(card))
            .collect()
    }

    fn take_game_action(&mut self, game: &mut Game) -> bool {
        if self.play_land(game) {
            return true;
        }

        let battlefield = self.combo_status(game, vec![Zone::Battlefield]);

        let castable = game.find_castable();

        if !self.is_storming && battlefield.cost_reducers < 2 {
            let cost_reducers = ["Sapphire Medallion", "Helm of Awakening"];
            if count_in_hand(game, &cost_reducers) > 0 {
                // Using petals for cost reducers seems worth it
                if self.cast_named(game, castable.clone(), "Lotus Petal") {
                    return true;
                }
            }

            for card_name in cost_reducers {
                if self.cast_named(game, castable.clone(), card_name) {
                    return true;
                }
            }
        }

        if !self.is_storming {
            // Is it time to start storming?
            let hand = self.combo_status(game, vec![Zone::Hand]);

            if battlefield.lands >= 2
                && (battlefield.cost_reducers >= 1 || battlefield.lands >= 5)
                && hand.cantrips >= 1
            {
                self.is_storming = true;
                debug!(
                    "[Turn {turn:002}][Strategy]: Time to start storming!",
                    turn = game.turn
                )
            }
        }

        if self.is_storming {
            // We might as well float all mana now to make casting untappers easy
            game.float_mana();

            // NOTE: `castable` needs to be always refreshed after floating mana, not optimal
            let mut castable = game.find_castable();

            for card_name in ["Lotus Petal", "Cloud of Faeries", "Turnabout"] {
                if self.cast_named(game, castable.clone(), card_name) {
                    return true;
                }
            }

            if battlefield.cloud_of_faeries > 0 {
                if self.cast_named(game, castable.clone(), "Snap") {
                    return true;
                }
            }

            // Check if there are enough brain freezes in hand for the win and cast them
            let brain_freezes = count_in_hand(game, &["Brain Freeze"]);
            let mut extras_from_storm = 0;
            for i in 0..brain_freezes {
                extras_from_storm += i + 1 * 3;
            }

            let total_milled = 3 * brain_freezes * game.storm + extras_from_storm;
            if game.opponent_library <= total_milled as i32 {
                if self.cast_named(game, castable.clone(), "Brain Freeze") {
                    return true;
                }
            }

            let priority_order = [
                "Meditate",
                "Frantic Search",
                "Impulse",
                "Words of Wisdom",
                "Sleight of hand",
                "Merchant Scroll",
                "Cunning Wish",
            ];

            for card_name in priority_order {
                if self.cast_named(game, castable.clone(), card_name) {
                    return true;
                }
            }

            // Cast anything else we can, cheapest first
            castable.sort_by(|(a, _), (b, _)| sort_by_cmc(a, b));

            if let Some((card_ref, payment)) = castable.first() {
                game.cast_spell(self, card_ref, payment.as_ref().unwrap(), None);
                return true;
            }
        } else {
            // Cast some of the non-premium cantrips to find cost reducers
            let priority_order = ["Impulse", "Sleight of Hand", "Words of Wisdom"];

            for card_name in priority_order {
                if self.cast_named(game, castable.clone(), card_name) {
                    return true;
                }
            }

            // Rather than discarding play something
            if game.game_objects.iter().filter(is_hand).count() > 7 {
                let priority_order = ["Lotus Petal", "Cloud of Faeries", "Merchant Scroll"];
                for card_name in priority_order {
                    if self.cast_named(game, castable.clone(), card_name) {
                        return true;
                    }
                }
            }
        }

        return false;
    }
}
