use std::collections::HashMap;
use std::rc::Rc;

use crate::card::{CardRef, CardType, SearchFilter};
use crate::deck::Decklist;
use crate::game::GameState;
use crate::strategy::Strategy;
use crate::utils::*;

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

pub struct PatternRector {}

impl PatternRector {
    fn cast_pattern_of_rebirth(&self, game: &mut GameState) -> bool {
        let castable = game.find_castable();

        let is_creature_on_battlefield = game
            .game_objects
            .iter()
            .any(|card| is_battlefield(&card) && is_creature(&card));
        let is_pattern_on_battlefield = game
            .game_objects
            .iter()
            .any(|card| is_battlefield(&card) && is_pattern(&card));

        let pattern_of_rebirth = castable.iter().find(|(card, _)| is_pattern(&card));

        if let Some((card_ref, payment)) = pattern_of_rebirth {
            if payment.is_some() && is_creature_on_battlefield && !is_pattern_on_battlefield {
                // Target non-sacrifice outlets over sac outlets
                let non_sac_creature = game
                    .game_objects
                    .iter()
                    .find(|card| is_battlefield(card) && is_creature(card) && !is_sac_outlet(card));

                let target = if let Some(creature) = non_sac_creature {
                    Rc::clone(creature)
                } else {
                    // Otherwise just cast it on a sac outlet
                    let sac_creature = game.game_objects.iter().find(|card| {
                        is_battlefield(card) && is_creature(card) && is_sac_outlet(card)
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
            .filter(|(c, _)| is_creature(&c))
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

        let creatures = game_objects.clone().filter(is_creature).count();
        let academy_rectors = game_objects.clone().filter(is_rector).count();
        let multi_use_sac_outlets = game_objects.clone().filter(is_sac_outlet).count();
        let patterns = game_objects.clone().filter(is_pattern).count();
        let lands = game_objects.clone().filter(is_land).count();

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

impl Strategy for PatternRector {
    fn decklist() -> Decklist {
        Decklist {
            maindeck: vec![
                ("Birds of Paradise", 4),
                ("Llanowar Elves", 3),
                ("Carrion Feeder", 4),
                ("Nantuko Husk", 3),
                ("Phyrexian Ghoul", 1),
                ("Pattern of Rebirth", 4),
                ("Academy Rector", 4),
                // ("Enlightened Tutor", 3),
                // ("Worldly Tutor", 3),
                ("Elvish Spirit Guide", 3),
                // ("Mesmeric Fiend", 3),
                ("Iridescent Drake", 1),
                ("Karmic Guide", 2),
                ("Caller of the Claw", 1),
                ("Body Snatcher", 1),
                ("Akroma, Angel of Wrath", 1),
                ("Volrath's Shapeshifter", 2),
                ("Worship", 1),
                ("Goblin Bombardment", 1),
                ("Cabal Therapy", 4),
                ("City of Brass", 4),
                ("Llanowar Wastes", 4),
                ("Yavimaya Coast", 2),
                ("Caves of Koilos", 1),
                ("Gemstone Mine", 2),
                ("Reflecting Pool", 1),
                ("Phyrexian Tower", 2),
                ("Forest", 2),
                ("Swamp", 1),
                ("Plains", 1),
            ],
            sideboard: vec![],
        }
    }

    fn is_win_condition_met(&self, game: &GameState) -> bool {
        // TODO: Make sure we still have the required combo pieces in library

        let status = self.combo_status(game, false, true);

        // Winning combinations:

        // 1) At least one sac outlet + Pattern of Rebirth on another
        if status.multi_use_sac_outlets >= 1
            && status.patterns >= 1
            && !status.pattern_on_sac_outlet
        {
            return true;
        }

        // 2) One sac outlet with pattern + one sac outlet without + Pattern of Rebirth on a sac outlet
        if status.multi_use_sac_outlets >= 2 && status.patterns >= 1 && status.pattern_on_sac_outlet
        {
            return true;
        }

        // 3) Sac outlet + Academy Rector + any redundant creature
        if status.multi_use_sac_outlets >= 1 && status.academy_rectors >= 1 && status.creatures >= 3
        {
            return true;
        }

        // 4) At least one Academy Rector + Pattern of Rebirth on a creature + Cabal Therapy in graveyard / Phyrexian Tower
        if status.academy_rectors >= 1 && status.patterns >= 1 && status.single_use_sac_outlets >= 1
        {
            return true;
        }

        // 5) At least two Academy Rectors + at least one single use sac outlet + at least three creatures total
        if status.academy_rectors >= 2
            && status.single_use_sac_outlets >= 1
            && status.creatures >= 3
        {
            return true;
        }

        // 6) At least two Academy Rectors + at least two single use sac outlets available
        // Sac first, get Pattern on second, sac the second, get Drake + Bombardment
        if status.academy_rectors >= 2 && status.single_use_sac_outlets >= 2 {
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

    fn select_best_card(&self, _game: &GameState, cards: HashMap<String, Vec<CardRef>>) -> Option<CardRef> {
        cards.values().flatten().cloned().next()
    }

    fn best_card_to_draw(&self, game: &GameState, search_filter: Option<SearchFilter>) -> &str {
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

        match search_filter {
            Some(SearchFilter::Creature) => {
                if is_pattern_attached_to_redundant_creature {
                    return "Carrion Feeder";
                }

                if status.academy_rectors == 0 && status.patterns == 0 {
                    return "Academy Rector";
                }

                if status.multi_use_sac_outlets == 0 {
                    return "Carrion Feeder";
                }

                if status.mana_sources < 4 {
                    return "Birds of Paradise";
                }

                "Academy Rector"
            }
            Some(SearchFilter::EnchantmentArtifact) => {
                if is_pattern_attached_to_redundant_creature {
                    return "Goblin Bombardment";
                }

                if status.academy_rectors == 0 && status.patterns == 0 {
                    return "Pattern of Rebirth";
                }

                if status.multi_use_sac_outlets == 0 {
                    return "Goblin Bombardment";
                }

                if status.mana_sources < 4 {
                    return "Lotus Petal";
                }

                "Pattern of Rebirth"
            }
            _ => unimplemented!(),
        }
    }

    fn worst_cards_in_hand(&self, game: &GameState, hand_size: usize) -> Vec<CardRef> {
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
            || self.cast_mana_dork(game)
            || self.cast_other_creature(game)
            || self.cast_others(game)
    }
}
