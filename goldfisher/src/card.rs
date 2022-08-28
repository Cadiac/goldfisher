use std::cell::RefCell;
use std::collections::{HashSet, HashMap};
use std::rc::Rc;

use crate::effect::Effect;
use crate::mana::{CostReduction, Mana};

pub type CardRef = Rc<RefCell<Card>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CardType {
    Creature,
    Enchantment,
    Artifact,
    Sorcery,
    Instant,
    Land,
}

impl Default for CardType {
    fn default() -> Self {
        CardType::Creature
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SubType {
    Creature(CreatureType),
    Land(LandType)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Zone {
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Exile,
    Outside,
}

pub const ZONES: &[Zone] = &[
    Zone::Library,
    Zone::Hand,
    Zone::Battlefield,
    Zone::Graveyard,
    Zone::Exile,
    Zone::Outside,
];

impl Default for Zone {
    fn default() -> Self {
        Zone::Library
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CreatureType {
    Harpy,
    Beast,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LandType {
    Plains,
    Island,
    Swamp,
    Mountain,
    Forest,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SearchFilter {
    Creature,
    Wish(Vec<CardType>),
    GreenCreature,
    EnchantmentArtifact,
    BlueInstant,
    Blue,
    Land(Vec<LandType>),
}

#[derive(Clone, Debug, Default)]
pub struct Card {
    pub name: String,
    pub card_types: HashSet<CardType>,
    pub sub_types: HashSet<SubType>,
    pub zone: Zone,
    pub cost: HashMap<Mana, i32>,
    pub produced_mana: HashMap<Mana, u32>,
    pub remaining_uses: Option<usize>,
    pub is_sac_outlet: bool,
    pub is_summoning_sick: bool,
    pub is_tapped: bool,
    pub is_haste: bool,
    pub on_resolve: Option<Effect>,
    pub attached_to: Option<CardRef>,
    pub cost_reduction: Option<CostReduction>,
}

impl Card {
    pub fn new(card_name: &str) -> Result<Card, String> {
        let name = card_name.to_owned();

        let card = match name.as_str() {
            "Llanowar Elves" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Green, 1)]),
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                ..Default::default()
            },
            "Veteran Explorer" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Green, 1)]),
                ..Default::default()
            },
            "Xantid Swarm" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Green, 1)]),
                ..Default::default()
            },
            "Fyndhorn Elves" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Green, 1)]),
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                ..Default::default()
            },
            "Birds of Paradise" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Green, 1)]),
                produced_mana: HashMap::from([
                    (Mana::White, 1),
                    (Mana::Blue, 1),
                    (Mana::Black, 1),
                    (Mana::Red, 1),
                    (Mana::Green, 1),
                ]),
                ..Default::default()
            },
            "Noble Hierarch" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Green, 1)]),
                produced_mana: HashMap::from([(Mana::White, 1), (Mana::Blue, 1), (Mana::Green, 1)]),
                ..Default::default()
            },
            "Carrion Feeder" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Black, 1)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Viscera Seer" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Black, 1)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Nantuko Husk" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Phyrexian Ghoul" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Pattern of Rebirth" => Card {
                name,
                card_types: HashSet::from([CardType::Enchantment]),
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 3)]),
                ..Default::default()
            },
            "Academy Rector" => Card {
                name: card_name.to_owned(),
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 3)]),
                ..Default::default()
            },
            "Mesmeric Fiend" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Iridescent Drake" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 3)]),
                ..Default::default()
            },
            "Karmic Guide" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::White, 2), (Mana::Colorless, 3)]),
                ..Default::default()
            },
            "Volrath's Shapeshifter" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Blue, 2), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Caller of the Claw" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Body Snatcher" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Black, 2), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Akroma, Angel of Wrath" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::White, 3), (Mana::Colorless, 5)]),
                ..Default::default()
            },
            "Phantom Nishoba" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::White, 1), (Mana::Green, 1), (Mana::Colorless, 5)]),
                ..Default::default()
            },
            "Worship" => Card {
                name,
                card_types: HashSet::from([CardType::Enchantment]),
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 3)]),
                ..Default::default()
            },
            "Pernicious Deed" => Card {
                name,
                card_types: HashSet::from([CardType::Enchantment]),
                cost: HashMap::from([(Mana::Green, 1), (Mana::Black, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Recurring Nightmare" => Card {
                name,
                card_types: HashSet::from([CardType::Enchantment]),
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Seal of Cleansing" => Card {
                name,
                card_types: HashSet::from([CardType::Enchantment]),
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "City of Solitude" => Card {
                name,
                card_types: HashSet::from([CardType::Enchantment]),
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Engineered Plague" => Card {
                name,
                card_types: HashSet::from([CardType::Enchantment]),
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Circle of Protection: Red" => Card {
                name,
                card_types: HashSet::from([CardType::Enchantment]),
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Warmth" => Card {
                name,
                card_types: HashSet::from([CardType::Enchantment]),
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Goblin Bombardment" => Card {
                name,
                card_types: HashSet::from([CardType::Enchantment]),
                cost: HashMap::from([(Mana::Red, 1), (Mana::Colorless, 1)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Altar of Dementia" => Card {
                name,
                card_types: HashSet::from([CardType::Artifact]),
                cost: HashMap::from([(Mana::Colorless, 2)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Cabal Therapy" => Card {
                name,
                card_types: HashSet::from([CardType::Sorcery]),
                cost: HashMap::from([(Mana::Black, 1)]),
                ..Default::default()
            },
            "Duress" => Card {
                name,
                card_types: HashSet::from([CardType::Sorcery]),
                cost: HashMap::from([(Mana::Black, 1)]),
                ..Default::default()
            },
            "Swords to Plowshares" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::White, 1)]),
                ..Default::default()
            },
            "Worldly Tutor" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Green, 1)]),
                on_resolve: Some(Effect::SearchAndPutTopOfLibrary(Some(
                    SearchFilter::Creature,
                ))),
                ..Default::default()
            },
            "Enlightened Tutor" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::White, 1)]),
                on_resolve: Some(Effect::SearchAndPutTopOfLibrary(Some(
                    SearchFilter::EnchantmentArtifact,
                ))),
                ..Default::default()
            },
            "Eladamri's Call" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::White, 1), (Mana::Green, 1)]),
                on_resolve: Some(Effect::SearchAndPutHand(Some(SearchFilter::Creature))),
                ..Default::default()
            },
            "Vindicate" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::White, 1), (Mana::Black, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Rofellos, Llanowar Emissary" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                // TODO: Actual produced mana
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                cost: HashMap::from([(Mana::Green, 2)]),
                ..Default::default()
            },
            "Wall of Roots" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                remaining_uses: Some(5),
                is_haste: true,
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Elvish Spirit Guide" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 2)]),
                remaining_uses: Some(1),
                ..Default::default()
            },
            "Lotus Petal" => Card {
                name,
                card_types: HashSet::from([CardType::Artifact]),
                produced_mana: HashMap::from([
                    (Mana::White, 1),
                    (Mana::Blue, 1),
                    (Mana::Black, 1),
                    (Mana::Red, 1),
                    (Mana::Green, 1),
                ]),
                cost: HashMap::new(),
                remaining_uses: Some(1),
                ..Default::default()
            },
            "Soul Warden" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::White, 1)]),
                ..Default::default()
            },
            "Unearth" => Card {
                name,
                card_types: HashSet::from([CardType::Sorcery]),
                cost: HashMap::from([(Mana::Black, 1)]),
                on_resolve: Some(Effect::Unearth),
                ..Default::default()
            },
            "Cavern Harpy" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                sub_types: HashSet::from([SubType::Creature(CreatureType::Harpy), SubType::Creature(CreatureType::Beast)]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Black, 1)]),
                on_resolve: Some(Effect::CavernHarpy),
                ..Default::default()
            },
            "Cloud of Faeries" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 1)]),
                on_resolve: Some(Effect::UntapLands(Some(2))),
                ..Default::default()
            },
            "Impulse" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 1)]),
                on_resolve: Some(Effect::Impulse(4)),
                ..Default::default()
            },
            "Living Wish" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 1)]),
                on_resolve: Some(Effect::SearchAndPutHand(Some(SearchFilter::Wish(vec![
                    CardType::Creature,
                    CardType::Land,
                ])))),
                ..Default::default()
            },
            "Cunning Wish" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 2)]),
                on_resolve: Some(Effect::SearchAndPutHand(Some(SearchFilter::Wish(vec![
                    CardType::Instant,
                ])))),
                ..Default::default()
            },
            "Ray of Revelation" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Intuition" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 2)]),
                on_resolve: Some(Effect::Intuition),
                ..Default::default()
            },
            "Raven Familiar" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 2)]),
                on_resolve: Some(Effect::Impulse(3)), // TODO: Separate effect
                ..Default::default()
            },
            "Wirewood Savage" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Aluren" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Green, 2), (Mana::Colorless, 2)]),
                cost_reduction: Some(CostReduction::Aluren),
                ..Default::default()
            },
            "Maggot Carrier" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                on_resolve: Some(Effect::DamageEach(1)),
                ..Default::default()
            },
            "Auramancer" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Monk Realist" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Plague Spitter" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Ravenous Baloth" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                sub_types: HashSet::from([SubType::Creature(CreatureType::Beast)]),
                cost: HashMap::from([(Mana::Green, 2), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Naturalize" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Crippling Fatigue" => Card {
                name,
                card_types: HashSet::from([CardType::Sorcery]),
                cost: HashMap::from([(Mana::Black, 2), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Uktabi Orangutan" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Bone Shredder" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Hydroblast" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 1)]),
                ..Default::default()
            },
            "Blue Elemental Blast" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 1)]),
                ..Default::default()
            },
            "Mana Short" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Words of Wisdom" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 1)]),
                on_resolve: Some(Effect::WordsOfWisdom),
                ..Default::default()
            },
            "Snap" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 1)]),
                on_resolve: Some(Effect::Snap),
                ..Default::default()
            },
            "Brain Freeze" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 1)]),
                on_resolve: Some(Effect::BrainFreeze),
                ..Default::default()
            },
            "Frantic Search" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 2)]),
                on_resolve: Some(Effect::FranticSearch),
                ..Default::default()
            },
            "Meditate" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 2)]),
                on_resolve: Some(Effect::Meditate),
                ..Default::default()
            },
            "Merchant Scroll" => Card {
                name,
                card_types: HashSet::from([CardType::Sorcery]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 1)]),
                on_resolve: Some(Effect::SearchAndPutHand(Some(SearchFilter::BlueInstant))),
                ..Default::default()
            },
            "Sleight of Hand" => Card {
                name,
                card_types: HashSet::from([CardType::Sorcery]),
                cost: HashMap::from([(Mana::Blue, 1)]),
                on_resolve: Some(Effect::Impulse(2)),
                ..Default::default()
            },
            "Helm of Awakening" => Card {
                name,
                card_types: HashSet::from([CardType::Artifact]),
                cost: HashMap::from([(Mana::Colorless, 2)]),
                cost_reduction: Some(CostReduction::All(Mana::Colorless, 1)),
                ..Default::default()
            },
            "Sapphire Medallion" => Card {
                name,
                card_types: HashSet::from([CardType::Artifact]),
                cost: HashMap::from([(Mana::Colorless, 2)]),
                cost_reduction: Some(CostReduction::Color(Mana::Blue, (Mana::Colorless, 1))),
                ..Default::default()
            },
            "Chain of Vapor" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 1)]),
                ..Default::default()
            },
            "Defense Grid" => Card {
                name,
                card_types: HashSet::from([CardType::Artifact]),
                cost: HashMap::from([(Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Tormod's Crypt" => Card {
                name,
                card_types: HashSet::from([CardType::Artifact]),
                cost: HashMap::new(),
                ..Default::default()
            },
            "Hurkyl's Recall" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Turnabout" => Card {
                name,
                card_types: HashSet::from([CardType::Instant]),
                cost: HashMap::from([(Mana::Blue, 2), (Mana::Colorless, 2)]),
                on_resolve: Some(Effect::UntapLands(None)),
                ..Default::default()
            },
            "City of Brass" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                produced_mana: HashMap::from([
                    (Mana::White, 1),
                    (Mana::Blue, 1),
                    (Mana::Black, 1),
                    (Mana::Red, 1),
                    (Mana::Green, 1),
                ]),
                ..Default::default()
            },
            "Llanowar Wastes" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                produced_mana: HashMap::from([
                    (Mana::Black, 1),
                    (Mana::Green, 1),
                    (Mana::Colorless, 1),
                ]),
                ..Default::default()
            },
            "Brushland" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                produced_mana: HashMap::from([
                    (Mana::White, 1),
                    (Mana::Green, 1),
                    (Mana::Colorless, 1),
                ]),
                ..Default::default()
            },
            "Yavimaya Coast" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                produced_mana: HashMap::from([
                    (Mana::Blue, 1),
                    (Mana::Green, 1),
                    (Mana::Colorless, 1),
                ]),
                ..Default::default()
            },
            "Caves of Koilos" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                produced_mana: HashMap::from([
                    (Mana::White, 1),
                    (Mana::Black, 1),
                    (Mana::Colorless, 1),
                ]),
                ..Default::default()
            },
            "Underground River" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                produced_mana: HashMap::from([
                    (Mana::Blue, 1),
                    (Mana::Black, 1),
                    (Mana::Colorless, 1),
                ]),
                ..Default::default()
            },
            "Gemstone Mine" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                remaining_uses: Some(3),
                produced_mana: HashMap::from([
                    (Mana::White, 1),
                    (Mana::Blue, 1),
                    (Mana::Black, 1),
                    (Mana::Red, 1),
                    (Mana::Green, 1),
                ]),
                ..Default::default()
            },
            "Reflecting Pool" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                // TODO: dynamically figure out what mana this produces
                produced_mana: HashMap::from([
                    (Mana::White, 1),
                    (Mana::Blue, 1),
                    (Mana::Black, 1),
                    (Mana::Red, 1),
                    (Mana::Green, 1),
                ]),
                ..Default::default()
            },
            "Phyrexian Tower" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                // TODO: the black mana from sac
                produced_mana: HashMap::from([(Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Ancient Tomb" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                produced_mana: HashMap::from([(Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Hickory Woodlot" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                produced_mana: HashMap::from([(Mana::Green, 2)]),
                is_tapped: true,
                remaining_uses: Some(2),
                ..Default::default()
            },
            "Dryad Arbor" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                is_summoning_sick: true,
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                ..Default::default()
            },
            "Plains" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Plains)]),
                produced_mana: HashMap::from([(Mana::White, 1)]),
                ..Default::default()
            },
            "Island" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Island)]),
                produced_mana: HashMap::from([(Mana::Blue, 1)]),
                ..Default::default()
            },
            "Swamp" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Swamp)]),
                produced_mana: HashMap::from([(Mana::Black, 1)]),
                ..Default::default()
            },
            "Mountain" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Mountain)]),
                produced_mana: HashMap::from([(Mana::Red, 1)]),
                ..Default::default()
            },
            "Forest" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Forest)]),
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                ..Default::default()
            },
            "Tundra" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Plains), SubType::Land(LandType::Island)]),
                produced_mana: HashMap::from([(Mana::Blue, 1), (Mana::White, 1)]),
                ..Default::default()
            },
            "Underground Sea" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Island), SubType::Land(LandType::Swamp)]),
                produced_mana: HashMap::from([(Mana::Blue, 1), (Mana::Black, 1)]),
                ..Default::default()
            },
            "Volcanic Island" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Island), SubType::Land(LandType::Mountain)]),
                produced_mana: HashMap::from([(Mana::Blue, 1), (Mana::Red, 1)]),
                ..Default::default()
            },
            "Tropical Island" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Island), SubType::Land(LandType::Forest)]),
                produced_mana: HashMap::from([(Mana::Blue, 1), (Mana::Green, 1)]),
                ..Default::default()
            },
            "Scrubland" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Plains), SubType::Land(LandType::Swamp)]),
                produced_mana: HashMap::from([(Mana::White, 1), (Mana::Black, 1)]),
                ..Default::default()
            },
            "Badlands" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Swamp), SubType::Land(LandType::Mountain)]),
                produced_mana: HashMap::from([(Mana::Black, 1), (Mana::Red, 1)]),
                ..Default::default()
            },
            "Bayou" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Swamp), SubType::Land(LandType::Forest)]),
                produced_mana: HashMap::from([(Mana::Black, 1), (Mana::Green, 1)]),
                ..Default::default()
            },
            "Plateau" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Plains), SubType::Land(LandType::Mountain)]),
                produced_mana: HashMap::from([(Mana::Red, 1), (Mana::White, 1)]),
                ..Default::default()
            },
            "Savannah" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Plains), SubType::Land(LandType::Forest)]),
                produced_mana: HashMap::from([(Mana::Green, 1), (Mana::White, 1)]),
                ..Default::default()
            },
            "Taiga" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                sub_types: HashSet::from([SubType::Land(LandType::Forest), SubType::Land(LandType::Mountain)]),
                produced_mana: HashMap::from([(Mana::Green, 1), (Mana::Red, 1)]),
                ..Default::default()
            },
            "Flooded Strand" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                on_resolve: Some(Effect::SearchAndPutBattlefield(Some(
                    SearchFilter::Land(vec![LandType::Plains, LandType::Island]),
                ))),
                ..Default::default()
            },
            "Marsh Flats" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                on_resolve: Some(Effect::SearchAndPutBattlefield(Some(
                    SearchFilter::Land(vec![LandType::Plains, LandType::Swamp]),
                ))),
                ..Default::default()
            },
            "Windswept Heath" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                on_resolve: Some(Effect::SearchAndPutBattlefield(Some(
                    SearchFilter::Land(vec![LandType::Plains, LandType::Forest]),
                ))),
                ..Default::default()
            },
            "Arid Mesa" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                on_resolve: Some(Effect::SearchAndPutBattlefield(Some(
                    SearchFilter::Land(vec![LandType::Plains, LandType::Mountain]),
                ))),
                ..Default::default()
            },
            "Polluted Delta" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                on_resolve: Some(Effect::SearchAndPutBattlefield(Some(
                    SearchFilter::Land(vec![LandType::Island, LandType::Swamp]),
                ))),
                ..Default::default()
            },
            "Scalding Tarn" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                on_resolve: Some(Effect::SearchAndPutBattlefield(Some(
                    SearchFilter::Land(vec![LandType::Island, LandType::Mountain]),
                ))),
                ..Default::default()
            },
            "Misty Rainforest" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                on_resolve: Some(Effect::SearchAndPutBattlefield(Some(
                    SearchFilter::Land(vec![LandType::Island, LandType::Forest]),
                ))),
                ..Default::default()
            },
            "Verdant Catacombs" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                on_resolve: Some(Effect::SearchAndPutBattlefield(Some(
                    SearchFilter::Land(vec![LandType::Swamp, LandType::Forest]),
                ))),
                ..Default::default()
            },
            "Bloodstained Mire" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                on_resolve: Some(Effect::SearchAndPutBattlefield(Some(
                    SearchFilter::Land(vec![LandType::Swamp, LandType::Mountain]),
                ))),
                ..Default::default()
            },
            "Wooded Foothills" => Card {
                name,
                card_types: HashSet::from([CardType::Land]),
                on_resolve: Some(Effect::SearchAndPutBattlefield(Some(
                    SearchFilter::Land(vec![LandType::Forest, LandType::Mountain]),
                ))),
                ..Default::default()
            },
            "Reveillark" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 4)]),
                // TODO: Effect
                ..Default::default()
            },
            "Body Double" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 4)]),
                // TODO: Effect
                ..Default::default()
            },
            "Protean Hulk" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Green, 2), (Mana::Colorless, 5)]),
                ..Default::default()
            },
            "Natural Order" => Card {
                name,
                card_types: HashSet::from([CardType::Sorcery]),
                cost: HashMap::from([(Mana::Green, 2), (Mana::Colorless, 2)]),
                on_resolve: Some(Effect::SearchAndPutBattlefield(Some(
                    SearchFilter::GreenCreature,
                ))),
                ..Default::default()
            },
            "Gitaxian Probe" => Card {
                name,
                card_types: HashSet::from([CardType::Sorcery]),
                // TODO: Phyrexian mana, but just pay life for now
                cost: HashMap::new(),
                on_resolve: Some(Effect::Draw(1)),
                ..Default::default()
            },
            "Mogg Fanatic" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([(Mana::Red, 1)]),
                ..Default::default()
            },
            "Progenitus" => Card {
                name,
                card_types: HashSet::from([CardType::Creature]),
                cost: HashMap::from([
                    (Mana::White, 2),
                    (Mana::Blue, 2),
                    (Mana::Black, 2),
                    (Mana::Red, 2),
                    (Mana::Green, 2),
                ]),
                ..Default::default()
            },
            name => {
                return Err(format!("unimplemented card: {name}"));
            }
        };

        Ok(card)
    }

    pub fn new_as_ref(name: &str) -> CardRef {
        Rc::new(RefCell::new(Card::new(name).unwrap()))
    }

    pub fn new_with_zone(name: &str, zone: Zone) -> CardRef {
        let mut card = Card::new(name).unwrap();
        card.zone = zone;
        Rc::new(RefCell::new(card))
    }
}
