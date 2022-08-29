use std::collections::HashMap;

use crate::card::{CardRef, CardType, Zone};
use crate::deck::Decklist;
use crate::game::{Game, Outcome, GameStatus};
use crate::strategy::Strategy;
use crate::utils::*;

pub const NAME: &str = "Legacy - Turbo Smog";
const DEFAULT_DECKLIST: &str = include_str!("../../resources/turbo-smog.txt");

struct ComboStatus {
    lands: usize,
    mana_sources: usize,
    chain_of_smogs: usize,
    sedgemoor_witches: usize,
    witherbloom_apprentices: usize,
    tutors: usize,
    cantrips: usize,
}

pub struct TurboSmog {
    is_wincon: bool
}

impl TurboSmog {
    pub fn new() -> Self {
        Self {
            is_wincon: false
        }
    }

    fn combo_status(&self, game: &Game, zones: Vec<Zone>) -> ComboStatus {
        let chain_of_smogs = game
            .game_objects
            .iter()
            .filter(|card| is_hand(card) && is_named(card, "Chain of Smog"))
            .count();

        let cantrips = game
            .game_objects
            .iter()
            .filter(|card| {
                is_hand(card) && is_named(card, "Brainstorm")
                    || is_named(card, "Preordain")
                    || is_named(card, "Ponder")
            })
            .count();

        let tutors = game
            .game_objects
            .iter()
            .filter(|card| {
                is_hand(card) && is_named(card, "Lim-Dûl's Vault")
                    || is_named(card, "Summoner's Pact")
            })
            .count();

        let game_objects = game
            .game_objects
            .iter()
            .filter(|card| zones.contains(&card.borrow().zone));

        let witherbloom_apprentices = game_objects
            .clone()
            .filter(|card| is_named(card, "Witherbloom Apprentice"))
            .count();

        let sedgemoor_witches = game_objects
            .clone()
            .filter(|card| is_named(card, "Sedgemoor Witch"))
            .count();

        let lands = game_objects
            .clone()
            .filter(|card| is_card_type(card, &CardType::Land))
            .count();

        let mana_sources = game_objects
            .clone()
            .filter(|card| is_card_type(card, &CardType::Land) || is_mana_source(card))
            .count();

        ComboStatus {
            lands,
            witherbloom_apprentices,
            sedgemoor_witches,
            chain_of_smogs,
            cantrips,
            tutors,
            mana_sources,
        }
    }
}

impl Strategy for TurboSmog {
    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn default_decklist(&self) -> Decklist {
        DEFAULT_DECKLIST.parse::<Decklist>().unwrap()
    }

    fn game_status(&self, _game: &Game) -> super::GameStatus {
        if self.is_wincon {
            return GameStatus::Finished(Outcome::Win)
        }

        GameStatus::Continue
    }

    fn is_keepable_hand(&self, game: &Game, mulligan_count: usize) -> bool {
        if mulligan_count >= 3 {
            // Just keep any hand with 4 cards
            return true;
        }

        let hand = self.combo_status(game, vec![Zone::Hand]);

        // The "perfect" hand
        if hand.chain_of_smogs >= 1
            && (hand.witherbloom_apprentices >= 1 || hand.sedgemoor_witches >= 1)
            && hand.mana_sources >= 2
        {
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

        if hand.cantrips == 0 && hand.tutors == 0 {
            // No cantrips or tutors, such mulligan
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

        if status.chain_of_smogs == 0 {
            let card = find_named(&cards, "Chain of Smog");
            if card.is_some() {
                return card;
            }
        }

        if status.witherbloom_apprentices == 0 {
            if let Some(card) = find_named(&cards, "Witherbloom Apprentice") {
                return Some(card);
            }
        }

        for name in [
            "Ponder",
            "Preordain",
            "Brainstorm",
            "Summoner's Pact",
            "Lotus Petal",
            "Dark Ritual",
            "Elvish Spirit Guide",
            "Sedgemoor Witch",
            "Lim-Dûl's Vault",
        ] {
            let card = find_named(&cards, name);
            if card.is_some() {
                return card;
            }
        }

        // Otherwise just pick anything
        cards.values().flatten().cloned().next()
    }

    fn take_game_action(&mut self, game: &mut Game) -> bool {
        if self.play_land(game) {
            return true;
        }

        let battlefield = self.combo_status(game, vec![Zone::Battlefield]);

        let castable = game.find_castable();
        if self.cast_named(game, castable.clone(), "Lotus Petal") {
            return true;
        }

        if battlefield.witherbloom_apprentices > 0 || battlefield.sedgemoor_witches > 0 {
            if self.cast_named(game, castable.clone(), "Chain of Smog") {
                self.is_wincon = true;
                return true;
            }
        }

        if battlefield.witherbloom_apprentices == 0 {
            if self.cast_named(game, castable.clone(), "Witherbloom Apprentice") {
                return true;
            }
        }

        if battlefield.sedgemoor_witches == 0 {
            if self.cast_named(game, castable.clone(), "Sedgemoor Witch") {
                return true;
            }
        }

        let priority_order = [
            "Summoner's Pact",
            "Ponder",
            "Preordain",
            "Brainstorm",
            "Lim-Dûl's Vault"
        ];

        for card_name in priority_order {
            if self.cast_named(game, castable.clone(), card_name) {
                return true;
            }
        }

        return false;
    }
}
