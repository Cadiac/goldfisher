use std::collections::HashMap;

use goldfisher::deck::create_deck;
use goldfisher::game::GameState;

fn main() {
    let deck = create_deck(vec![
        ("Birds of Paradise", 4),
        ("Llanowar Elves", 3),
        ("Carrion Feeder", 4),
        ("Nantuko Husk", 3),
        ("Phyrexian Ghoul", 1),
        ("Pattern of Rebirth", 4),
        ("Academy Rector", 4),
        ("Mesmeric Fiend", 3),
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
    ]);

    let mut win_statistics: HashMap<usize, usize> = HashMap::new();
    let simulated_games = 100;

    for _ in 0..simulated_games {
        let mut game = GameState::new(deck.clone());

        game.find_starting_hand();

        loop {
            println!("========================================");

            game.advance_turn();
            game.untap();
            game.draw();
            game.print_game_state();

            game.play_land();

            game.cast_pattern_of_rebirths();
            game.cast_rectors();

            if game.mana_sources_count() >= 4 {
                game.cast_sac_outlets();
                game.cast_mana_dorks();
            } else {
                game.cast_mana_dorks();
                game.cast_sac_outlets();
            }
            game.cast_redundant_creatures();
            game.cast_others();

            // Do we have it?
            if game.is_win_condition_met() {
                println!("========================================");
                println!(" Won the game on turn {turn}!", turn = game.turn);
                println!("========================================");
                game.print_game_state();

                *win_statistics.entry(game.turn).or_insert(0) += 1;
                break;
            }

            // If not, take another turn
            game.cleanup();
        }
        println!("");
    }

    let mut wins_by_turn = win_statistics.iter().collect::<Vec<_>>();
    wins_by_turn.sort();

    println!("========================================");
    println!("Wins per turn after {simulated_games} games:");
    for (turn, wins) in wins_by_turn {
        println!(
            "Turn {turn:002}: {wins} wins ({percentage:.1}%).",
            percentage = 100.0 * *wins as f32 / simulated_games as f32
        );
    }
}
