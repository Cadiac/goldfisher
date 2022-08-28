use clap::Parser;
use env_logger::Env;
use std::collections::HashMap;
use std::error::Error;
use std::fs;

use rayon::prelude::*;

use goldfisher::deck::{Decklist};
use goldfisher::game::{Game, GameResult, Outcome};
use goldfisher::strategy::{DeckStrategy, Strategy};

#[macro_use]
extern crate log;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ArgDeckStrategy {
    PatternCombo,
    Aluren,
    FranticStorm,
}

impl From<ArgDeckStrategy> for DeckStrategy {
    fn from(other: ArgDeckStrategy) -> DeckStrategy {
        match other {
            ArgDeckStrategy::PatternCombo => DeckStrategy::PatternCombo,
            ArgDeckStrategy::Aluren => DeckStrategy::Aluren,
            ArgDeckStrategy::FranticStorm => DeckStrategy::FranticStorm,
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of games to simulate
    #[clap(short, long, value_parser, default_value_t = 100)]
    games: usize,

    /// Print game actions debug output (slow)
    #[clap(short, long, action)]
    verbose: bool,

    /// The name of the deck strategy to use.
    #[clap(short, long, value_enum)]
    strategy: ArgDeckStrategy,

    /// Path to custom decklist file
    #[clap(short, long)]
    decklist: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();
    init_logger(cli.verbose);

    let mut win_statistics: HashMap<usize, usize> = HashMap::new();
    let mut loss_statistics: HashMap<usize, usize> = HashMap::new();
    let simulated_games = cli.games;

    let decklist: Decklist = match cli.decklist {
        Some(path) => fs::read_to_string(path)?.parse()?,
        None => {
            let strategy: Box<dyn Strategy> = goldfisher::strategy::from_enum(&cli.strategy.clone().into());
            strategy.default_decklist()
        }
    };

    let results: Vec<_> = (0..simulated_games)
        .into_par_iter()
        .map(|_| {
            let mut strategy: Box<dyn Strategy> =
                goldfisher::strategy::from_enum(&cli.strategy.clone().into());

            let mut game = match Game::new(&decklist) {
                Ok(game) => game,
                Err(err) => {
                    panic!("failed to initialize game: {err:?}");
                }
            };

            game.run(&mut strategy)
        })
        .collect();

    let mut mulligans = Vec::with_capacity(simulated_games);

    for GameResult { result, turn, mulligan_count, output: _ } in results {
        match result {
            Outcome::Win => {
                *win_statistics.entry(turn).or_insert(0) += 1;
                mulligans.push(mulligan_count);
            }
            Outcome::Lose | Outcome::Draw => {
                *loss_statistics.entry(turn).or_insert(0) += 1;
                mulligans.push(mulligan_count);
            }
        }
    }

    let mut wins_by_turn = win_statistics.iter().collect::<Vec<_>>();
    let mut losses_by_turn = loss_statistics.iter().collect::<Vec<_>>();

    wins_by_turn.sort();
    losses_by_turn.sort();

    let total_wins: usize = wins_by_turn.iter().map(|(_, wins)| *wins).sum();
    let average_turn = wins_by_turn
        .iter()
        .map(|(turn, wins)| *turn * *wins)
        .sum::<usize>() as f32
        / total_wins as f32;

    let average_mulligans = mulligans.iter().sum::<usize>() as f32 / mulligans.len() as f32;

    info!("=======================[ RESULTS ]==========================");
    info!("                   Average turn: {average_turn:.2}");
    info!("                 Average mulligans: {average_mulligans:.2}");
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

    Ok(())
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
