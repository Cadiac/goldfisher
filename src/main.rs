use clap::Parser;
use env_logger::Env;
use std::collections::HashMap;

use goldfisher::game::GameState;
use goldfisher::strategy::pattern_rector::PatternRector;

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

    let decklist = vec![
        ("Birds of Paradise", 4),
        ("Llanowar Elves", 3),
        ("Carrion Feeder", 4),
        ("Nantuko Husk", 3),
        ("Phyrexian Ghoul", 1),
        ("Pattern of Rebirth", 4),
        ("Academy Rector", 4),
        // ("Enlightened Tutor", 3),
        ("Worldly Tutor", 3),
        // ("Elvish Spirit Guide", 3),
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
    let simulated_games = cli.games;
    let strategy = PatternRector {};

    for _ in 0..simulated_games {
        debug!("====================[ START OF GAME ]=======================");
        let mut game = GameState::new(decklist.clone());

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
