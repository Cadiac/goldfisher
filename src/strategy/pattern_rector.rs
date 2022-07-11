use std::rc::Rc;

use crate::card::{CardRef, CardType, SearchFilter};
use crate::game::GameState;
use crate::strategy::Strategy;
use crate::utils::*;

pub struct PatternRector {}

impl PatternRector {
    pub fn best_land_in_hand(&self, game: &GameState) -> Option<CardRef> {
        let mut lands_in_hand = game
            .game_objects
            .iter()
            .filter(|card| is_hand(card) && is_land(card))
            .collect::<Vec<_>>();

        lands_in_hand.sort_by(|a, b| sort_by_produced_mana(a, b));

        // Play the one that produces most colors
        // TODO: Play the one that produces most cards that could be played
        lands_in_hand.last().map(|card| (*card).clone())
    }

    pub fn play_land(&self, game: &mut GameState) -> bool {
        if game.available_land_drops > 0 {
            if let Some(land) = self.best_land_in_hand(game) {
                game.play_land(land);
                return true;
            }
        }
        false
    }

    pub fn cast_pattern_of_rebirth(&self, game: &mut GameState) -> bool {
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

    pub fn cast_academy_rector(&self, game: &mut GameState) -> bool {
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

    pub fn cast_mana_dork(&self, game: &mut GameState) -> bool {
        let castable = game.find_castable();

        let mut mana_dorks = castable
            .iter()
            .filter(|(card, _)| is_mana_dork(&card))
            .collect::<Vec<_>>();

        // Cast the one that produces most colors
        mana_dorks.sort_by(|(a, _), (b, _)| sort_by_produced_mana(a, b));

        if let Some((card_ref, payment)) = mana_dorks.last() {
            game.cast_spell(self, card_ref, payment.as_ref().unwrap(), None);
            return true;
        }

        false
    }

    pub fn cast_sac_outlet(&self, game: &mut GameState) -> bool {
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

    pub fn cast_other_creature(&self, game: &mut GameState) -> bool {
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

    pub fn cast_others(&self, game: &mut GameState) -> bool {
        let mut castable = game.find_castable();

        // Cast the cheapest first
        castable.sort_by(|(a, _), (b, _)| sort_by_cmc(a, b));

        if let Some((card_ref, payment)) = castable.first() {
            game.cast_spell(self, card_ref, payment.as_ref().unwrap(), None);
            return true;
        }

        return false;
    }
}

impl Strategy for PatternRector {
    fn is_win_condition_met(&self, game: &GameState) -> bool {
        let creatures = game
            .game_objects
            .iter()
            .filter(|card| is_battlefield(card) && is_creature(card))
            .count();

        let rectors = game
            .game_objects
            .iter()
            .filter(|card| is_battlefield(card) && is_rector(card))
            .count();

        let sac_outlets = game
            .game_objects
            .iter()
            .filter(|card| is_battlefield(card) && is_sac_outlet(card))
            .count();

        let cabal_therapies_in_graveyard = game
            .game_objects
            .iter()
            .filter(|card| is_graveyard(card) && is_named(card, "Cabal Therapy"))
            .count();

        let patterns = game
            .game_objects
            .iter()
            .filter(|card| is_battlefield(card) && is_pattern(card))
            .count();

        let is_pattern_attached_to_redundant_creature = game.game_objects.iter().any(|card| {
            if is_battlefield(&card) && is_pattern(&card) {
                let card = card.borrow();

                match &card.attached_to {
                    Some(target) => {
                        if target.borrow().is_sac_outlet {
                            // Make sure we have a redundant sac outlet if pattern is attached to one
                            sac_outlets > 1
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

        let untapped_phyrexian_tower = game.game_objects.iter().any(|card| {
            is_battlefield(&card) && is_named(&card, "Phyrexian Tower") && !is_tapped(&card)
        });

        // Winning combinations:
        // 1) Sac outlet + any redundant creature + Pattern of Rebirth on that creature
        if sac_outlets >= 1 && is_pattern_attached_to_redundant_creature {
            return true;
        }

        // 2) Sac outlet + Academy Rector + any redundant creature
        if sac_outlets >= 1 && rectors >= 1 && creatures >= 3 {
            return true;
        }

        // 3) At least one Academy Rector + Pattern of Rebirth on a creature + Cabal Therapy in graveyard / Phyrexian Tower
        // TODO: Make sure we still have Goblin Bombardment in library
        if rectors >= 1
            && patterns >= 1
            && (cabal_therapies_in_graveyard >= 1 || untapped_phyrexian_tower)
        {
            return true;
        }

        // 4) At least two Academy Rectors + Cabal Therapy in graveyard / Phyrexian Tower + at least three creatures total
        if rectors >= 2
            && creatures >= 3
            && (cabal_therapies_in_graveyard >= 1 || untapped_phyrexian_tower)
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

        let hand = game.game_objects.iter().filter(is_hand);

        let mut is_pattern_in_hand = false;
        let mut is_rector_in_hand = false;
        let mut is_sac_outlet_in_hand = false;

        let mut creatures_count = 0;
        let mut lands_count = 0;
        let mut mana_dorks_count = 0;

        for card in hand {
            let card = card.borrow();

            if card.name == "Pattern of Rebirth" {
                is_pattern_in_hand = true;
            }

            if card.name == "Academy Rector" {
                is_rector_in_hand = true;
            }

            if card.is_sac_outlet {
                is_sac_outlet_in_hand = true;
            }

            if card.card_type == CardType::Creature {
                creatures_count += 1;
            }

            if card.card_type == CardType::Land {
                lands_count += 1;
            }

            if card.card_type == CardType::Creature && !card.produced_mana.is_empty() {
                mana_dorks_count += 1;
            }
        }

        if lands_count == 0 {
            // Always mulligan zero land hands
            return false;
        }

        if lands_count >= 6 {
            // Also mulligan too land heavy hands
            return false;
        }

        if lands_count == 1 && mana_dorks_count <= 1 {
            // One landers with just max one mana dork get automatically mulliganed too
            return false;
        }

        // Having a rector/pattern and sac outlet in hand is always good
        if (is_pattern_in_hand || is_rector_in_hand) && is_sac_outlet_in_hand {
            return true;
        }

        if (is_pattern_in_hand || is_rector_in_hand || is_sac_outlet_in_hand) && creatures_count > 0
        {
            // If we have already taken any mulligans this should be good enough
            if mulligan_count > 0 {
                return true;
            }

            // At full hand one of the combo pieces with reasonable mana is a keep
            if lands_count + mana_dorks_count >= 3 {
                return true;
            }
        }

        // Otherwise take a mulligan
        false
    }

    fn best_card_to_draw(&self, game: &GameState, search_filter: Option<SearchFilter>) -> &str {
        let creatures = game
            .game_objects
            .iter()
            .filter(|card| (is_battlefield(card) || is_hand(card)) && is_creature(card))
            .count();

        let rectors = game
            .game_objects
            .iter()
            .filter(|card| (is_battlefield(card) || is_hand(card)) && is_rector(card))
            .count();

        let sac_outlets = game
            .game_objects
            .iter()
            .filter(|card| (is_battlefield(card) || is_hand(card)) && is_sac_outlet(card))
            .count();

        let patterns = game
            .game_objects
            .iter()
            .filter(|card| (is_battlefield(card) || is_hand(card)) && is_pattern(card))
            .count();

        let mana_sources = game
            .game_objects
            .iter()
            .filter(|card| (is_battlefield(card) || is_hand(card)) && is_mana_source(card))
            .count();

        let is_pattern_attached_to_redundant_creature = game.game_objects.iter().any(|card| {
            if is_battlefield(&card) && is_pattern(&card) {
                let card = card.borrow();

                match &card.attached_to {
                    Some(target) => {
                        if target.borrow().is_sac_outlet {
                            // Make sure we have a redundant sac outlet if pattern is attached to one
                            sac_outlets > 1
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

                if sac_outlets >= 1 && creatures >= 2 && (rectors == 0 && patterns == 0) {
                    return "Academy Rector";
                }

                if rectors == 0 && patterns == 0 {
                    return "Academy Rector";
                }

                if sac_outlets == 0 {
                    return "Carrion Feeder";
                }

                if mana_sources < 4 {
                    return "Birds of Paradise";
                }

                return "Academy Rector";
            }
            Some(SearchFilter::EnchantmentArtifact) => {
                if is_pattern_attached_to_redundant_creature {
                    return "Goblin Bombardment";
                }

                if sac_outlets >= 1 && creatures >= 2 && (rectors == 0 && patterns == 0) {
                    return "Pattern of Rebirth";
                }

                if rectors == 0 && patterns == 0 {
                    return "Pattern of Rebirth";
                }

                if sac_outlets == 0 {
                    return "Goblin Bombardment";
                }

                return "Pattern of Rebirth";
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

        lands.sort_by(sort_by_produced_mana);
        sac_outlets.sort_by(sort_by_cmc);

        // First take a balanced mix of lands and combo pieces
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
