use goldfisher::deck::{create_deck};
use goldfisher::game::{GameState};

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

    let mut game = GameState::new(deck);

    // Take the opening 7
    game.draw_n(7);

    game.advance_turn();

    loop {
        game.untap();

        game.draw();

        game.print_game_state();

        game.play_land();

        game.cast_pattern_of_rebirths();

        game.cast_sac_outlets();

        game.cast_creatures();

        // N. Do we have it?
        // TODO: test if its attached to the only sac outlet
        if game.is_combo_ready() {
            println!("Won the game on turn {turn}!", turn = game.turn);
            game.print_game_state();
            return;
        }

        // If not, take another turn
        game.cleanup();

        game.advance_turn();
    }
}
