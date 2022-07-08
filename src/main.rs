use std::cell::RefCell;
use std::rc::Rc;

use goldfisher::card::{Card, Zone};
use goldfisher::deck::{create_deck};
use goldfisher::game;

fn main() {
    let mut deck = create_deck(vec![
        ("Birds of Paradise", 4),
        ("Llanowar Elves", 2),
        ("Carrion Feeder", 4),
        ("Nantuko Husk", 3),
        ("Phyrexian Ghoul", 1),
        ("Pattern of Rebirth", 4),
        ("Academy Rector", 4),
        ("Forest", 12),
        ("Swamp", 4),
        ("Plains", 4),
    ]);

    deck.shuffle();

    let mut turn = 1;
    let mut game_objects: Vec<Rc<RefCell<Card>>> = Vec::new();

    let is_first_player = true;

    // Take the opening 7
    for _ in 0..7 {
        if let Some(mut card) = deck.draw() {
            card.zone = Zone::Hand;
            game_objects.push(Rc::new(RefCell::new(card)))
        }
    }

    loop {
        game::untap(&game_objects);

        game::draw(turn, is_first_player, &mut deck, &mut game_objects);

        game::print_game_state(&game_objects, &deck, turn);

        game::play_land(&game_objects);

        game::cast_pattern_of_rebirths(&game_objects);

        game::cast_sac_outlets(&game_objects);

        game::cast_creatures(&game_objects);

        // N. Do we have it?
        // TODO: test if its attached to the only sac outlet
        if game::is_combo_ready(&game_objects) {
            println!("Won the game on turn {turn}!");
            game::print_game_state(&game_objects, &deck, turn);
            return;
        }

        // If not, take another turn
        game::cleanup(&game_objects);

        turn += 1;
    }
}
