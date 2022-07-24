use log::debug;
use std::collections::HashMap;
use std::rc::Rc;

use crate::card::{CardRef, CardType, Zone};
use crate::deck::Decklist;
use crate::game::GameState;
use crate::mana::Mana;
use crate::strategy::Strategy;
use crate::utils::*;

use super::GameStatus;

const DEFAULT_DECKLIST: &str = include_str!("../../resources/pattern-hulk.txt");

struct ComboStatus {
    mana_sources: usize,
    lands: usize,
    creatures: usize,
    academy_rectors: usize,
    multi_use_sac_outlets: usize,
    single_use_sac_outlets: usize,
    patterns: usize,
    pattern_on_sac_outlet: bool,
}

pub struct PatternHulk {}

impl PatternHulk {
    fn cast_pattern_of_rebirth(&self, game: &mut GameState) -> bool {
        let castable = game.find_castable();

        let is_creature_on_battlefield = game
            .game_objects
            .iter()
            .any(|card| is_battlefield(&card) && is_card_type(&card, CardType::Creature));
        let is_pattern_on_battlefield = game
            .game_objects
            .iter()
            .any(|card| is_battlefield(&card) && is_pattern(&card));

        let pattern_of_rebirth = castable.iter().find(|(card, _)| is_pattern(&card));

        if let Some((card_ref, payment)) = pattern_of_rebirth {
            if payment.is_some() && is_creature_on_battlefield && !is_pattern_on_battlefield {
                // Target non-sacrifice outlets over sac outlets
                let non_sac_creature = game.game_objects.iter().find(|card| {
                    is_battlefield(card)
                        && is_card_type(card, CardType::Creature)
                        && !is_sac_outlet(card)
                });

                let target = if let Some(creature) = non_sac_creature {
                    Rc::clone(creature)
                } else {
                    // Otherwise just cast it on a sac outlet
                    let sac_creature = game.game_objects.iter().find(|card| {
                        is_battlefield(card)
                            && is_card_type(card, CardType::Creature)
                            && is_sac_outlet(card)
                    });

                    Rc::clone(sac_creature.unwrap())
                };

                game.cast_spell(self, card_ref, payment.as_ref().unwrap(), Some(target));

                return true;
            }
        }

        false
    }

    fn cast_academy_rector(&self, game: &mut GameState) -> bool {
        let castable = game.find_castable();

        let rector = castable.iter().find(|(card, _)| is_rector(&card));
        let is_pattern_on_battlefield = game
            .game_objects
            .iter()
            .any(|card| is_battlefield(&card) && is_pattern(&card));

        if let Some((card_ref, payment)) = rector {
            if payment.is_some() && !is_pattern_on_battlefield {
                game.cast_spell(self, card_ref, payment.as_ref().unwrap(), None);
                return true;
            }
        }

        false
    }

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

        let veteran_explorer = castable
            .iter()
            .find(|(card, _)| is_named(&card, "Veteran Explorer"));

        if let Some((card_ref, payment)) = veteran_explorer {
            game.cast_spell(self, card_ref, payment.as_ref().unwrap(), None);
            return true;
        }

        false
    }

    fn sac_veteran_explorer(&self, game: &mut GameState, veteran_explorer: CardRef) {
        veteran_explorer.borrow_mut().zone = Zone::Graveyard;

        for _ in 0..2 {
            let basics = game
                .game_objects
                .iter()
                .filter(|card| is_library(card) && is_basic(card))
                .cloned()
                .collect();
            if let Some(land) = self.select_best(game, group_by_name(basics)) {
                land.borrow_mut().zone = Zone::Battlefield;
                debug!(
                    "[Turn {turn:002}][Action]: Searched for \"{card_name}\" with \"Veteran Explorer\" dies trigger.",
                    card_name = land.borrow().name,
                    turn = game.turn
                );
            }
        }
    }

    fn ramp_with_veteran_explorer(&self, game: &mut GameState) -> bool {
        let veteran_explorer = match game
            .game_objects
            .iter()
            .find(|card| is_battlefield(card) && is_named(card, "Veteran Explorer"))
            .cloned()
        {
            Some(card) => card,
            None => return false,
        };

        if let Some(sac_outlet) = game
            .game_objects
            .iter()
            .find(|card| is_battlefield(&card) && is_sac_outlet(&card))
            .cloned()
        {
            debug!(
                "[Turn {turn:002}][Action]: Sacrificing \"Veteran Explorer\" with \"{card_name}\".",
                card_name = sac_outlet.borrow().name,
                turn = game.turn
            );
            self.sac_veteran_explorer(game, veteran_explorer);
            return true;
        }

        if let Some(cabal_therapy) = game
            .game_objects
            .iter()
            .find(|card| is_graveyard(card) && is_named(card, "Cabal Therapy"))
            .cloned()
        {
            debug!(
                "[Turn {turn:002}][Action]: Sacrificing \"Veteran Explorer\" with \"{card_name}\".",
                card_name = cabal_therapy.borrow().name,
                turn = game.turn
            );
            self.sac_veteran_explorer(game, veteran_explorer);
            cabal_therapy.borrow_mut().zone = Zone::Exile;
            return true;
        }

        if let Some(phyrexian_tower) = game
            .game_objects
            .iter()
            .find(|card| {
                is_battlefield(card) && is_named(card, "Phyrexian Tower") && !is_tapped(card)
            })
            .cloned()
        {
            debug!(
                "[Turn {turn:002}][Action]: Sacrificing \"Veteran Explorer\" with \"{card_name}\".",
                card_name = phyrexian_tower.borrow().name,
                turn = game.turn
            );
            self.sac_veteran_explorer(game, veteran_explorer);
            phyrexian_tower.borrow_mut().is_tapped = true;
            *game.floating_mana.entry(Mana::Black).or_insert(0) += 2;
            return true;
        }

        false
    }

    fn cast_sac_outlet(&self, game: &mut GameState) -> bool {
        let castable = game.find_castable();

        let mut sac_outlets = castable
            .iter()
            .filter(|(card, _)| is_sac_outlet(&card))
            .collect::<Vec<_>>();

        // Cast the cheapest first
        sac_outlets.sort_by(|(a, _), (b, _)| sort_by_cmc(a, b));

        if let Some((card_ref, payment)) = sac_outlets.first() {
            game.cast_spell(self, card_ref, payment.as_ref().unwrap(), None);
            return true;
        }

        false
    }

    fn cast_other_creature(&self, game: &mut GameState) -> bool {
        let castable = game.find_castable();

        let mut creatures = castable
            .iter()
            .filter(|(c, _)| is_card_type(&c, CardType::Creature))
            .collect::<Vec<_>>();

        // Cast the cheapest creatures first
        creatures.sort_by(|(a, _), (b, _)| sort_by_cmc(a, b));

        if let Some((card_ref, payment)) = creatures.first() {
            game.cast_spell(self, card_ref, payment.as_ref().unwrap(), None);
            return true;
        }

        false
    }

    fn cast_others(&self, game: &mut GameState) -> bool {
        let mut castable = game.find_castable();

        // Cast the cheapest first
        castable.sort_by(|(a, _), (b, _)| sort_by_cmc(a, b));

        if let Some((card_ref, payment)) = castable.first() {
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

        let creatures = game_objects
            .clone()
            .filter(|card| is_card_type(card, CardType::Creature))
            .count();
        let academy_rectors = game_objects.clone().filter(is_rector).count();
        let multi_use_sac_outlets = game_objects.clone().filter(is_sac_outlet).count();
        let patterns = game_objects.clone().filter(is_pattern).count();
        let lands = game_objects
            .clone()
            .filter(|card| is_card_type(card, CardType::Land))
            .count();

        let mana_sources = game_objects
            .clone()
            .filter(|card| is_mana_source(card) && !is_single_use_mana(card))
            .count();

        let pattern_on_sac_outlet = game.game_objects.iter().any(|card| {
            if is_battlefield(&card) && is_pattern(&card) {
                let card = card.borrow();

                match &card.attached_to {
                    Some(target) => return target.borrow().is_sac_outlet,
                    None => false,
                }
            } else {
                false
            }
        });

        let cabal_therapies_in_graveyard = game
            .game_objects
            .iter()
            .filter(|card| is_graveyard(card) && is_named(card, "Cabal Therapy"))
            .count();

        let untapped_phyrexian_towers = game
            .game_objects
            .iter()
            .filter(|card| {
                is_battlefield(card) && is_named(card, "Phyrexian Tower") && !is_tapped(card)
            })
            .count();

        let single_use_sac_outlets = cabal_therapies_in_graveyard + untapped_phyrexian_towers;

        ComboStatus {
            lands,
            mana_sources,
            creatures,
            academy_rectors,
            multi_use_sac_outlets,
            single_use_sac_outlets,
            patterns,
            pattern_on_sac_outlet,
        }
    }
}

impl Strategy for PatternHulk {
    fn default_decklist(&self) -> Decklist {
        DEFAULT_DECKLIST.parse::<Decklist>().unwrap()
    }

    fn game_status(&self, game: &GameState) -> super::GameStatus {
        if game.life_total <= 0 {
            debug!(
                "[Turn {turn:002}][Game]: Out of life points, lost the game!",
                turn = game.turn
            );
            return GameStatus::Lose(game.turn);
        }

        let status = self.combo_status(game, false, true);

        let mut count_in_library: HashMap<&str, usize> = HashMap::new();
        let combo_pieces = [
            "Body Snatcher",
            "Iridescent Drake",
            "Pattern of Rebirth",
            "Karmic Guide",
            "Volrath's Shapeshifter",
            "Academy Rector",
            "Goblin Bombardment",
            "Akroma, Angel of Wrath",
            "Caller of the Claw",
        ];
        for name in combo_pieces {
            let count = game
                .game_objects
                .iter()
                .filter(|card| is_library(&card) && is_named(&card, name))
                .count();

            *count_in_library.entry(name).or_insert(0) = count
        }

        let is_goblin_bombardment_on_battlefield = game
            .game_objects
            .iter()
            .any(|card| is_battlefield(&card) && is_named(&card, "Goblin Bombardment"));

        let is_goblin_bombardment_in_hand = game
            .game_objects
            .iter()
            .any(|card| is_hand(&card) && is_named(&card, "Goblin Bombardment"));

        // Make sure required combo pieces are still in library
        // NOTE: This is not be 100% accurate, and is probably missing some lines that
        // involve just playing out the cards from hand.
        // TODO: Handle at least cases where we have the bombardment in hand but we just haven't cast it.
        let simple_kill_available = is_goblin_bombardment_on_battlefield
            && *count_in_library.get("Iridescent Drake").unwrap() >= 1
            && (*count_in_library.get("Volrath's Shapeshifter").unwrap()
                + *count_in_library.get("Karmic Guide").unwrap()
                + *count_in_library.get("Body Snatcher").unwrap()
                >= 2);

        let main_kill_available = (*count_in_library.get("Volrath's Shapeshifter").unwrap()
            + *count_in_library.get("Karmic Guide").unwrap()
            + *count_in_library.get("Body Snatcher").unwrap()
            >= 3)
            && (*count_in_library.get("Iridescent Drake").unwrap() >= 1
                || *count_in_library.get("Body Snatcher").unwrap() >= 1)
            && *count_in_library.get("Academy Rector").unwrap() >= 1
            && *count_in_library.get("Pattern of Rebirth").unwrap() >= 1
            && *count_in_library.get("Goblin Bombardment").unwrap() >= 1;

        let backup_kill_available = *count_in_library.get("Volrath's Shapeshifter").unwrap() >= 2
            && (*count_in_library.get("Karmic Guide").unwrap()
                + *count_in_library.get("Body Snatcher").unwrap()
                >= 2)
            && *count_in_library.get("Academy Rector").unwrap() >= 1
            && *count_in_library.get("Pattern of Rebirth").unwrap() >= 1
            && *count_in_library.get("Akroma, Angel of Wrath").unwrap() >= 1
            && *count_in_library.get("Caller of the Claw").unwrap() >= 1;

        // TODO: This doesn't seem to be accurate
        if !simple_kill_available && !main_kill_available && !backup_kill_available && !is_goblin_bombardment_in_hand {
            debug!(
                "[Turn {turn:002}][Game]: Can't combo anymore, lost the game!",
                turn = game.turn
            );
            return GameStatus::Lose(game.turn);
        }

        // Winning combinations:

        // TODO: Treat Bombardment + Karmic Guide loop as a wincon.

        // TODO: Add a flag to toggle treating a non-summoning sick Carrion Feeded / Nantuko Husk attacker as wincon

        // 1) At least one sac outlet + Pattern of Rebirth on another
        if status.multi_use_sac_outlets >= 1
            && status.patterns >= 1
            && !status.pattern_on_sac_outlet
        {
            return GameStatus::Win(game.turn);
        }

        // 2) One sac outlet with pattern + one sac outlet without + Pattern of Rebirth on a sac outlet
        if status.multi_use_sac_outlets >= 2 && status.patterns >= 1 && status.pattern_on_sac_outlet
        {
            return GameStatus::Win(game.turn);
        }

        // 3) Sac outlet + Academy Rector + any redundant creature
        if status.multi_use_sac_outlets >= 1 && status.academy_rectors >= 1 && status.creatures >= 3
        {
            return GameStatus::Win(game.turn);
        }

        // 4) At least one Academy Rector + Pattern of Rebirth on a creature + Cabal Therapy in graveyard / Phyrexian Tower
        if status.academy_rectors >= 1 && status.patterns >= 1 && status.single_use_sac_outlets >= 1
        {
            return GameStatus::Win(game.turn);
        }

        // 5) At least two Academy Rectors + at least one single use sac outlet + at least three creatures total
        if status.academy_rectors >= 2
            && status.single_use_sac_outlets >= 1
            && status.creatures >= 3
        {
            return GameStatus::Win(game.turn);
        }

        // 6) At least two Academy Rectors + at least two single use sac outlets available
        // Sac first, get Pattern on second, sac the second, get Drake + Bombardment
        if status.academy_rectors >= 2 && status.single_use_sac_outlets >= 2 {
            return GameStatus::Win(game.turn);
        }

        GameStatus::Continue
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

        // Having a rector/pattern and sac outlet in hand is always good
        if (status.patterns >= 1 || status.academy_rectors >= 1)
            && status.multi_use_sac_outlets >= 1
        {
            return true;
        }

        if (status.patterns >= 1
            || status.academy_rectors >= 1
            || status.multi_use_sac_outlets >= 1)
            && status.creatures > 0
        {
            // If we have already taken two mulligans this should be good enough
            if mulligan_count > 1 {
                return true;
            }

            // At full hand with one of the combo pieces with is only a keep with fast mana
            // NOTE: Apparently it is better to just mulligan these hands always
            // if status.mana_sources >= 3 && status.fast_mana > 0 {
            //     return true;
            // }
        }

        // Otherwise take a mulligan
        false
    }

    fn select_best(
        &self,
        game: &GameState,
        cards: HashMap<String, Vec<CardRef>>,
    ) -> Option<CardRef> {
        let status = self.combo_status(game, true, true);

        let is_pattern_attached_to_redundant_creature = game.game_objects.iter().any(|card| {
            if is_battlefield(&card) && is_pattern(&card) {
                let card = card.borrow();

                match &card.attached_to {
                    Some(target) => {
                        if target.borrow().is_sac_outlet {
                            // Make sure we have a redundant sac outlet if pattern is attached to one
                            status.multi_use_sac_outlets >= 2
                        } else {
                            true
                        }
                    }
                    None => false,
                }
            } else {
                false
            }
        });

        if is_pattern_attached_to_redundant_creature {
            for name in [
                "Carrion Feeder",
                "Goblin Bombardment",
                "Nantuko Husk",
                "Phyrexian Ghoul",
            ] {
                let card = find_named(&cards, name);
                if card.is_some() {
                    return card;
                }
            }
        }

        if status.academy_rectors == 0 && status.patterns == 0 {
            for name in ["Academy Rector", "Pattern of Rebirth"] {
                let card = find_named(&cards, name);
                if card.is_some() {
                    return card;
                }
            }
        }

        if status.multi_use_sac_outlets == 0 {
            for name in [
                "Carrion Feeder",
                "Goblin Bombardment",
                "Nantuko Husk",
                "Phyrexian Ghoul",
            ] {
                let card = find_named(&cards, name);
                if card.is_some() {
                    return card;
                }
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
            } else {
                for name in ["Birds of Paradise", "Lotus Petal", "Wall of Roots"] {
                    let card = find_named(&cards, name);
                    if card.is_some() {
                        return card;
                    }
                }
            }
        }

        for name in ["Academy Rector", "Pattern of Rebirth"] {
            let card = find_named(&cards, name);
            if card.is_some() {
                return card;
            }
        }

        for name in ["Swamp", "Plains", "Forest", "Island", "Mountain"] {
            let card = find_named(&cards, name);
            if card.is_some() {
                return card;
            }
        }

        cards.values().flatten().cloned().next()
    }

    fn discard_to_hand_size(&self, game: &GameState, hand_size: usize) -> Vec<CardRef> {
        let mut ordered_hand = Vec::new();
        let mut lands = Vec::with_capacity(7);
        let mut patterns_or_rectors = Vec::with_capacity(7);
        let mut mana_dorks = Vec::with_capacity(7);
        let mut sac_outlets = Vec::with_capacity(7);
        let mut redundant_cards = Vec::with_capacity(7);

        let hand = game.game_objects.iter().filter(is_hand);

        for card in hand {
            let c = card.borrow();

            if c.card_type == CardType::Land {
                lands.push(card.clone());
            } else if c.name == "Pattern of Rebirth" || c.name == "Academy Rector" {
                patterns_or_rectors.push(card.clone());
            } else if c.is_sac_outlet {
                sac_outlets.push(card.clone());
            } else if c.card_type == CardType::Creature && !c.produced_mana.is_empty() {
                mana_dorks.push(card.clone());
            } else {
                redundant_cards.push(card.clone());
            }
        }

        lands.sort_by(sort_by_best_mana_to_play);
        sac_outlets.sort_by(sort_by_cmc);

        // First keep a balanced mix of lands and combo pieces
        // Prefer lands that produce the most colors of mana (sorted to the end of the iter)
        let mut lands_iter = lands.iter().rev();
        for _ in 0..2 {
            if let Some(card) = lands_iter.next() {
                ordered_hand.push(card);
            }
        }

        let mut patterns_or_rectors_iter = patterns_or_rectors.iter();
        for _ in 0..1 {
            if let Some(card) = patterns_or_rectors_iter.next() {
                ordered_hand.push(card);
            }
        }

        let mut sac_outlets_iter = sac_outlets.iter();
        for _ in 0..1 {
            if let Some(card) = sac_outlets_iter.next() {
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
        for card in patterns_or_rectors_iter {
            ordered_hand.push(card);
        }
        for card in sac_outlets_iter {
            ordered_hand.push(card);
        }
        for card in redundant_cards.iter() {
            ordered_hand.push(card);
        }

        ordered_hand
            .iter()
            .skip(hand_size)
            .map(|card| Rc::clone(card))
            .collect()
    }

    fn take_game_action(&self, game: &mut GameState) -> bool {
        self.play_land(game)
            || self.cast_pattern_of_rebirth(game)
            || self.cast_academy_rector(game)
            || self.cast_sac_outlet(game)
            || self.ramp_with_veteran_explorer(game)
            || self.cast_mana_dork(game)
            || self.cast_other_creature(game)
            || self.cast_others(game)
    }
}
