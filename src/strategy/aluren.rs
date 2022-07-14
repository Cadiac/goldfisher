use std::rc::Rc;

use crate::card::{CardRef, CardType, SearchFilter};
use crate::deck::Decklist;
use crate::game::GameState;
use crate::strategy::Strategy;
use crate::utils::*;

struct ComboStatus {
    mana_sources: usize,
    lands: usize,
    alurens: usize,
    cavern_harpies: usize,
    wirewood_savages: usize,
    raven_familiars: usize,
    soul_wardens: usize,
    maggot_carrier: bool,
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
        let soul_wardens = game_objects
            .clone()
            .filter(|card| card.borrow().name == "Soul Warden")
            .count();

        let maggot_carrier = game_objects
            .clone()
            .any(|card| card.borrow().name == "Maggot Carrier");

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
            wirewood_savages,
            raven_familiars,
            soul_wardens,
            maggot_carrier,
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
        let status = self.combo_status(game, false, true);

        // Aluren on the battlefield + Cavern Harpy on the battlefield
        // + Soul warden on the battlefield OR life total is at least 40
        if status.alurens >= 1
            && status.maggot_carrier
            && status.cavern_harpies >= 1
            && (status.soul_wardens >= 1 || game.life_total >= 40)
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
        if status.alurens >= 1 && (status.cavern_harpies >= 1 || (status.raven_familiars >= 1 || status.wirewood_savages >= 1)) {
            return true;
        }

        // If we have already taken a mulligans this should be good enough
        if  status.alurens >= 1 && mulligan_count > 0 {
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

                "Aluren"
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
        self.play_land(game) || self.cast_mana_dork(game) || self.cast_others(game)
    }
}
