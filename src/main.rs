use clap::Parser;
use env_logger::Env;
use std::collections::HashMap;

use goldfisher::game::{GameState, GameStatus};
use goldfisher::strategy::{Strategy};
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
    let mut loss_statistics: HashMap<usize, usize> = HashMap::new();
    let simulated_games = cli.games;

    let strategy = PatternRector {};
    // let strategy = Aluren {};

    for _ in 0..simulated_games {
        match simulate_game(&strategy) {
            GameStatus::Continue => panic!("stuck game"),
            GameStatus::Win(turn) => *win_statistics.entry(turn).or_insert(0) += 1,
            GameStatus::Lose(turn) => *loss_statistics.entry(turn).or_insert(0) += 1,
            GameStatus::Draw(turn) => *loss_statistics.entry(turn).or_insert(0) += 1,
        }
    }

    let mut wins_by_turn = win_statistics.iter().collect::<Vec<_>>();
    let mut losses_by_turn = loss_statistics.iter().collect::<Vec<_>>();

    wins_by_turn.sort();
    losses_by_turn.sort();

    info!("=======================[ RESULTS ]==========================");
    info!("              Wins per turn after {simulated_games} games:");
    info!("============================================================");

    let mut cumulative = 0.0;
    for (turn, wins) in wins_by_turn {
        let win_percentage = 100.0 * *wins as f32 / simulated_games as f32;
        cumulative += win_percentage;
        info!("Turn {turn:002}: {wins} wins ({win_percentage:.1}%) - cumulative {cumulative:.1}%");
    }

    let mut loss_cumulative = 0.0;
    for (turn, losses) in losses_by_turn {
        let loss_percentage = 100.0 * *losses as f32 / simulated_games as f32;
        loss_cumulative += loss_percentage;
        info!("Turn {turn:002}: {losses} losses ({loss_percentage:.1}%) - cumulative {loss_cumulative:.1}%");
    }
}

fn simulate_game(strategy: &impl Strategy) -> GameStatus {
    debug!("====================[ START OF GAME ]=======================");
    let mut game = GameState::new(PatternRector::decklist());

    game.find_starting_hand(strategy);

    let result = loop {
        game.begin_turn();

        debug!(
            "======================[ TURN {turn:002} ]===========================",
            turn = game.turn
        );

        game.untap();

        match game.draw() {
            GameStatus::Continue => (),
            result => break result
        }

        game.print_game_state();

        match game.take_game_actions(strategy) {
            GameStatus::Continue => (),
            result => break result
        }

        game.cleanup(strategy);
    };

    debug!("=====================[ END OF GAME ]========================");
    debug!(
        "                 Won the game on turn {turn}!",
        turn = game.turn
    );
    debug!("============================================================");
    game.print_game_state();

    result
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
