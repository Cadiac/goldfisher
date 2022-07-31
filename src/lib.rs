use std::collections::HashMap;
use gloo_worker::{HandlerId, Worker, WorkerScope};

use goldfisher::deck::Decklist;
use goldfisher::game::{Game, GameResult};
use goldfisher::strategy::{Strategy, pattern_hulk, aluren};

#[derive(Debug)]
pub enum Msg<T> {
    Respond { output: T, id: HandlerId },
}

pub struct Goldfish {}

impl Worker for Goldfish {
    type Input = (String, Decklist, usize);

    type Message = Msg<HashMap<(GameResult, usize), usize>>;

    type Output = HashMap<(GameResult, usize), usize>;

    fn create(_scope: &WorkerScope<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message) {
        let Msg::Respond { output, id } = msg;
        scope.respond(id, output);
    }

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, who: HandlerId) {
        let (strategy_name, decklist, simulations) = msg;

        let mut output = HashMap::new();
        let strategy: Box<dyn Strategy> = match strategy_name.as_str() {
            pattern_hulk::NAME => Box::new(pattern_hulk::PatternHulk {}),
            aluren::NAME => Box::new(aluren::Aluren {}),
            _ => unimplemented!()
        };

        for _ in 0..simulations {
            let mut game = Game::new(&decklist);
            let (result, turn) = game.run(&strategy);
            *output.entry((result, turn)).or_insert(0) += 1;
        }

        scope.send_message(Msg::Respond { output, id: who });
    }
}
