use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{RefCell};

use crate::mana::{Mana};

pub type CardRef = Rc<RefCell<Card>>;

#[derive(Clone, Debug, PartialEq)]
pub enum CardType {
    Creature,
    Enchantment,
    Artifact,
    Sorcery,
    Land,
}

impl Default for CardType {
    fn default() -> Self {
        CardType::Creature
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Zone {
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Exile
}

impl Default for Zone {
    fn default() -> Self {
        Zone::Library
    }
}

#[derive(Clone, Debug, Default)]
pub struct Card {
    pub name: String,
    pub card_type: CardType,
    pub zone: Zone,
    pub cost: HashMap<Mana, usize>,
    pub produced_mana: HashMap<Mana, usize>,
    pub is_sac_outlet: bool,
    pub is_rector: bool,
    pub is_pattern: bool,
    pub is_summoning_sick: bool,
    pub is_tapped: bool,
    pub attached_to: Option<CardRef>,
}
