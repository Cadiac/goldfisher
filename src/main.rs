use std::collections::HashMap;

use goldfisher::game::GameState;

#[macro_use]
extern crate log;

use env_logger::Env;

fn main() {
    init_logger();

    let decklist = vec![
        ("Birds of Paradise", 4),
        ("Llanowar Elves", 3),
        ("Carrion Feeder", 4),
        ("Nantuko Husk", 3),
        ("Phyrexian Ghoul", 1),
        ("Pattern of Rebirth", 4),
        ("Academy Rector", 4),
        ("Worldly Tutor", 3),
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
    ];

    let mut win_statistics: HashMap<usize, usize> = HashMap::new();
    let simulated_games = 500_000;

    for _ in 0..simulated_games {
        debug!("====================[ START OF GAME ]=======================");
        let mut game = GameState::new(decklist.clone());

        game.find_starting_hand();

        loop {
            game.advance_turn();

            debug!(
                "======================[ TURN {turn:002} ]===========================",
                turn = game.turn
            );

            game.untap();
            game.draw();
            game.print_game_state();

            game.play_land();
            if game.is_win_condition_met() {
                break;
            }

            game.cast_pattern_of_rebirths();
            if game.is_win_condition_met() {
                break;
            }

            game.cast_rectors();
            if game.is_win_condition_met() {
                break;
            }

            if game.mana_sources_count() >= 4 {
                game.cast_sac_outlets();
                if game.is_win_condition_met() {
                    break;
                }

                game.cast_mana_dorks();
                if game.is_win_condition_met() {
                    break;
                }
            } else {
                game.cast_mana_dorks();
                if game.is_win_condition_met() {
                    break;
                }

                game.cast_sac_outlets();
                if game.is_win_condition_met() {
                    break;
                }
            }

            game.cast_redundant_creatures();
            if game.is_win_condition_met() {
                break;
            }

            game.cast_others();
            if game.is_win_condition_met() {
                break;
            }

            game.cleanup();
        }

        debug!("=====================[ END OF GAME ]========================");
        debug!("                 Won the game on turn {turn}!", turn = game.turn);
        debug!("============================================================");
        game.print_game_state();

        *win_statistics.entry(game.turn).or_insert(0) += 1;
    }

    let mut wins_by_turn = win_statistics.iter().collect::<Vec<_>>();
    wins_by_turn.sort();

    info!("=======================[ RESULTS ]==========================");
    info!("              Wins per turn after {simulated_games} games:");
    info!("============================================================");

    let mut cumulative = 0.0;
    for (turn, wins) in wins_by_turn {
        let win_percentage = 100.0 * *wins as f32 / simulated_games as f32;
        cumulative += win_percentage;
        info!("Turn {turn:002}: {wins} wins ({win_percentage:.1}%) - cumulative {cumulative:.1}%");
    }
}

fn init_logger() {
    env_logger::Builder::from_env(
        Env::default()
            .filter_or("LOG_LEVEL", "debug")
            .write_style_or("LOG_STYLE", "always"),
    )
    .format_timestamp(None)
    .format_module_path(false)
    .init();
}
