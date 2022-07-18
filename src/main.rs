use clap::Parser;
use env_logger::Env;
use std::collections::HashMap;

use goldfisher::game::GameState;
use goldfisher::strategy::Strategy;
use goldfisher::strategy::pattern_rector::PatternRector;
// use goldfisher::strategy::aluren::Aluren;

#[macro_use]
extern crate log;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of games to simulate
    #[clap(short, long, value_parser, default_value_t = 100)]
    games: u32,

    #[clap(short, long, action)]
    verbose: bool,
}

fn main() {
    let cli = Args::parse();
    init_logger(cli.verbose);

    let mut win_statistics: HashMap<usize, usize> = HashMap::new();
    let simulated_games = cli.games;

    let strategy = PatternRector {};
    // let strategy = Aluren {};

    for _ in 0..simulated_games {
        debug!("====================[ START OF GAME ]=======================");
        let mut game = GameState::new(PatternRector::decklist());

        game.find_starting_hand(&strategy);

        loop {
            game.begin_turn();

            debug!(
                "======================[ TURN {turn:002} ]===========================",
                turn = game.turn
            );

            game.untap();
            game.draw();
            game.print_game_state();

            // Take game actions until we no longer can or the game has ended
            if game.take_game_actions(&strategy) {
                break;
            }

            game.cleanup(&strategy);
        }

        debug!("=====================[ END OF GAME ]========================");
        debug!(
            "                 Won the game on turn {turn}!",
            turn = game.turn
        );
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

fn init_logger(verbose: bool) {
    let default_level = if verbose { "debug" } else { "info" };

    env_logger::Builder::from_env(
        Env::default()
            .filter_or("LOG_LEVEL", default_level)
            .write_style_or("LOG_STYLE", "always"),
    )
    .format_timestamp(None)
    .format_module_path(false)
    .init();
}
