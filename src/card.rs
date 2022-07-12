use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::mana::Mana;

pub type CardRef = Rc<RefCell<Card>>;

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum Zone {
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Exile,
}

impl Default for Zone {
    fn default() -> Self {
        Zone::Library
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SearchFilter {
    Creature,
    EnchantmentArtifact,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    SearchAndPutHand(Option<SearchFilter>),
    SearchAndPutTopOfLibrary(Option<SearchFilter>),
    SearchAndPutBattlefield(Option<SearchFilter>),
}

#[derive(Clone, Debug, Default)]
pub struct Card {
    pub name: String,
    pub card_type: CardType,
    pub zone: Zone,
    pub cost: HashMap<Mana, usize>,
    pub produced_mana: HashMap<Mana, usize>,
    pub is_elvish_spirit_guide: bool,
    pub is_sac_outlet: bool,
    pub is_summoning_sick: bool,
    pub is_tapped: bool,
    pub on_resolve: Option<Effect>,
    pub attached_to: Option<CardRef>,
}

impl Card {
    pub fn new(card_name: &str) -> Card {
        let name = card_name.to_owned();

        match name.as_str() {
            "Llanowar Elves" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Green, 1)]),
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                ..Default::default()
            },
            "Fyndhorn Elves" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Green, 1)]),
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                ..Default::default()
            },
            "Birds of Paradise" => Card {
                name,
                card_type: CardType::Creature,
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
            "Carrion Feeder" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Black, 1)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Nantuko Husk" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Phyrexian Ghoul" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Pattern of Rebirth" => Card {
                name,
                card_type: CardType::Enchantment,
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 3)]),
                ..Default::default()
            },
            "Academy Rector" => Card {
                name: card_name.to_owned(),
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 3)]),
                ..Default::default()
            },
            "Mesmeric Fiend" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Iridescent Drake" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 3)]),
                ..Default::default()
            },
            "Karmic Guide" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::White, 2), (Mana::Colorless, 3)]),
                ..Default::default()
            },
            "Volrath's Shapeshifter" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Blue, 2), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Caller of the Claw" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Body Snatcher" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 3)]),
                ..Default::default()
            },
            "Akroma, Angel of Wrath" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::White, 3), (Mana::Colorless, 5)]),
                ..Default::default()
            },
            "Worship" => Card {
                name,
                card_type: CardType::Enchantment,
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 3)]),
                ..Default::default()
            },
            "Goblin Bombardment" => Card {
                name,
                card_type: CardType::Enchantment,
                cost: HashMap::from([(Mana::Red, 1), (Mana::Colorless, 1)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Altar of Dementia" => Card {
                name,
                card_type: CardType::Artifact,
                cost: HashMap::from([(Mana::Colorless, 2)]),
                is_sac_outlet: true,
                ..Default::default()
            },
            "Cabal Therapy" => Card {
                name,
                card_type: CardType::Sorcery,
                cost: HashMap::from([(Mana::Black, 1)]),
                ..Default::default()
            },
            "Duress" => Card {
                name,
                card_type: CardType::Sorcery,
                cost: HashMap::from([(Mana::Black, 1)]),
                ..Default::default()
            },
            "Worldly Tutor" => Card {
                name,
                card_type: CardType::Instant,
                cost: HashMap::from([(Mana::Green, 1)]),
                on_resolve: Some(Effect::SearchAndPutTopOfLibrary(Some(
                    SearchFilter::Creature,
                ))),
                ..Default::default()
            },
            "Enlightened Tutor" => Card {
                name,
                card_type: CardType::Instant,
                cost: HashMap::from([(Mana::White, 1)]),
                on_resolve: Some(Effect::SearchAndPutTopOfLibrary(Some(
                    SearchFilter::EnchantmentArtifact,
                ))),
                ..Default::default()
            },
            "Eladamri's Call" => Card {
                name,
                card_type: CardType::Instant,
                cost: HashMap::from([(Mana::White, 1), (Mana::Green, 1)]),
                on_resolve: Some(Effect::SearchAndPutHand(Some(
                    SearchFilter::Creature,
                ))),
                ..Default::default()
            },
            "Vindicate" => Card {
                name,
                card_type: CardType::Instant,
                cost: HashMap::from([(Mana::White, 1), (Mana::Black, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Rofellos, Llanowar Emissary" => Card {
                name,
                card_type: CardType::Creature,
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                cost: HashMap::from([(Mana::Green, 2)]),
                ..Default::default()
            },
            "Wall of Roots" => Card {
                name,
                card_type: CardType::Creature,
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Elvish Spirit Guide" => Card {
                name,
                card_type: CardType::Creature,
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 2)]),
                is_elvish_spirit_guide: true,
                ..Default::default()
            },
            "City of Brass" => Card {
                name,
                card_type: CardType::Land,
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
                card_type: CardType::Land,
                produced_mana: HashMap::from([
                    (Mana::Black, 1),
                    (Mana::Green, 1),
                    (Mana::Colorless, 1),
                ]),
                ..Default::default()
            },
            "Brushland" => Card {
                name,
                card_type: CardType::Land,
                produced_mana: HashMap::from([
                    (Mana::White, 1),
                    (Mana::Green, 1),
                    (Mana::Colorless, 1),
                ]),
                ..Default::default()
            },
            "Yavimaya Coast" => Card {
                name,
                card_type: CardType::Land,
                produced_mana: HashMap::from([
                    (Mana::Blue, 1),
                    (Mana::Green, 1),
                    (Mana::Colorless, 1),
                ]),
                ..Default::default()
            },
            "Caves of Koilos" => Card {
                name,
                card_type: CardType::Land,
                produced_mana: HashMap::from([
                    (Mana::White, 1),
                    (Mana::Black, 1),
                    (Mana::Colorless, 1),
                ]),
                ..Default::default()
            },
            "Gemstone Mine" => Card {
                name,
                card_type: CardType::Land,
                // TODO: max number of uses
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
                card_type: CardType::Land,
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
                card_type: CardType::Land,
                // TODO: the black mana from sac
                produced_mana: HashMap::from([(Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Plains" => Card {
                name,
                card_type: CardType::Land,
                produced_mana: HashMap::from([(Mana::White, 1)]),
                ..Default::default()
            },
            "Island" => Card {
                name,
                card_type: CardType::Land,
                produced_mana: HashMap::from([(Mana::White, 1)]),
                ..Default::default()
            },
            "Swamp" => Card {
                name,
                card_type: CardType::Land,
                produced_mana: HashMap::from([(Mana::Black, 1)]),
                ..Default::default()
            },
            "Mountain" => Card {
                name,
                card_type: CardType::Land,
                produced_mana: HashMap::from([(Mana::Black, 1)]),
                ..Default::default()
            },
            "Forest" => Card {
                name,
                card_type: CardType::Land,
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                ..Default::default()
            },
            "Ancient Tomb" => Card {
                name,
                card_type: CardType::Land,
                produced_mana: HashMap::from([(Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Hickory Woodlot" => Card {
                name,
                card_type: CardType::Land,
                produced_mana: HashMap::from([(Mana::Green, 2)]),
                ..Default::default()
            },
            "Taiga" => Card {
                name,
                card_type: CardType::Land,
                produced_mana: HashMap::from([(Mana::Green, 1), (Mana::Red, 1)]),
                ..Default::default()
            },
            "Scrubland" => Card {
                name,
                card_type: CardType::Land,
                produced_mana: HashMap::from([(Mana::White, 1), (Mana::Black, 1)]),
                ..Default::default()
            },
            name => unimplemented!("{}", name),
        }
    }

    pub fn new_as_ref(name: &str) -> CardRef {
        Rc::new(RefCell::new(Card::new(name)))
    }
}
