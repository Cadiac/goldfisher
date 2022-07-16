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
    LivingWish,
    EnchantmentArtifact,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    SearchAndPutHand(Option<SearchFilter>),
    SearchAndPutTopOfLibrary(Option<SearchFilter>),
    SearchAndPutBattlefield(Option<SearchFilter>),
    Impulse(usize),
    CavernHarpy,
    Unearth,
    UntapLands(usize)
}

#[derive(Clone, Debug, Default)]
pub struct Card {
    pub name: String,
    pub card_type: CardType,
    pub zone: Zone,
    pub cost: HashMap<Mana, usize>,
    pub produced_mana: HashMap<Mana, usize>,
    pub remaining_uses: Option<usize>,
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
                on_resolve: Some(Effect::SearchAndPutHand(Some(SearchFilter::Creature))),
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
                // TODO: Actual produced mana
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                cost: HashMap::from([(Mana::Green, 2)]),
                ..Default::default()
            },
            "Wall of Roots" => Card {
                name,
                card_type: CardType::Creature,
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                remaining_uses: Some(5),
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Elvish Spirit Guide" => Card {
                name,
                card_type: CardType::Creature,
                produced_mana: HashMap::from([(Mana::Green, 1)]),
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 2)]),
                remaining_uses: Some(1),
                ..Default::default()
            },
            "Lotus Petal" => Card {
                name,
                card_type: CardType::Artifact,
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
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::White, 1)]),
                ..Default::default()
            },
            "Unearth" => Card {
                name,
                card_type: CardType::Sorcery,
                cost: HashMap::from([(Mana::Black, 1)]),
                on_resolve: Some(Effect::Unearth),
                ..Default::default()
            },
            "Cavern Harpy" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Black, 1)]),
                on_resolve: Some(Effect::CavernHarpy),
                ..Default::default()
            },
            "Cloud of Faeries" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 1)]),
                on_resolve: Some(Effect::UntapLands(2)),
                ..Default::default()
            },
            "Impulse" => Card {
                name,
                card_type: CardType::Instant,
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 1)]),
                on_resolve: Some(Effect::Impulse(4)),
                ..Default::default()
            },
            "Living Wish" => Card {
                name,
                card_type: CardType::Instant,
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 1)]),
                on_resolve: Some(Effect::SearchAndPutHand(Some(SearchFilter::LivingWish))),
                ..Default::default()
            },
            "Ray of Revelation" => Card {
                name,
                card_type: CardType::Instant,
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Intuition" => Card {
                name,
                card_type: CardType::Instant,
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 2)]),
                on_resolve: Some(Effect::SearchAndPutHand(None)),
                ..Default::default()
            },
            "Raven Familiar" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Blue, 1), (Mana::Colorless, 2)]),
                on_resolve: Some(Effect::Impulse(3)), // TODO: Separate effect
                ..Default::default()
            },
            "Wirewood Savage" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Aluren" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Green, 2), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Maggot Carrier" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Auramancer" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Monk Realist" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::White, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Plague Spitter" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Naturalize" => Card {
                name,
                card_type: CardType::Instant,
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Crippling Fatigue" => Card {
                name,
                card_type: CardType::Sorcery,
                cost: HashMap::from([(Mana::Black, 2), (Mana::Colorless, 1)]),
                ..Default::default()
            },
            "Uktabi Orangutan" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Green, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Bone Shredder" => Card {
                name,
                card_type: CardType::Creature,
                cost: HashMap::from([(Mana::Black, 1), (Mana::Colorless, 2)]),
                ..Default::default()
            },
            "Hydroblast" => Card {
                name,
                card_type: CardType::Instant,
                cost: HashMap::from([(Mana::Blue, 1)]),
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
            "Underground River" => Card {
                name,
                card_type: CardType::Land,
                produced_mana: HashMap::from([
                    (Mana::Blue, 1),
                    (Mana::Black, 1),
                    (Mana::Colorless, 1),
                ]),
                ..Default::default()
            },
            "Gemstone Mine" => Card {
                name,
                card_type: CardType::Land,
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
                is_tapped: true,
                remaining_uses: Some(2),
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

    pub fn new_with_zone(name: &str, zone: Zone) -> CardRef {
        let mut card = Card::new(name);
        card.zone = zone;
        Rc::new(RefCell::new(card))
    }
}
