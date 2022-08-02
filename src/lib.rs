use gloo_worker::{HandlerId, Worker, WorkerScope};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;

use goldfisher::deck::Decklist;
use goldfisher::game::{Game, GameResult};
use goldfisher::strategy::{aluren, pattern_hulk, Strategy};

const MAX_BATCH_SIZE: usize = 250;

#[derive(Debug)]
pub enum Msg<T> {
    Respond { output: T, id: HandlerId },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    InProgress(usize, usize, Vec<(GameResult, usize)>),
    Complete(usize, Vec<(GameResult, usize)>),
}

pub struct Goldfish {}

impl Goldfish {
    fn run_simulations(
        strategy: &Rc<Box<dyn Strategy>>,
        decklist: &Decklist,
        batch_size: usize,
    ) -> Vec<(GameResult, usize)> {
        let mut results = Vec::new();

        for _ in 0..batch_size {
            let mut game = Game::new(&decklist);
            let result = game.run(strategy);
            results.push(result);
        }

        results
    }
}

impl Worker for Goldfish {
    type Input = (String, Decklist, usize);

    type Message = Msg<Status>;

    type Output = Status;

    fn create(_scope: &WorkerScope<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message) {
        let Msg::Respond { output, id } = msg;
        scope.respond(id, output);
    }

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId) {
        let (strategy_name, decklist, total_simulations) = msg;

        let strategy: Rc<Box<dyn Strategy>> = match strategy_name.as_str() {
            pattern_hulk::NAME => Rc::new(Box::new(pattern_hulk::PatternHulk {})),
            aluren::NAME => Rc::new(Box::new(aluren::Aluren {})),
            _ => unimplemented!(),
        };

        let mut progress = 0;
        while progress < total_simulations {
            let batch_size = if progress + MAX_BATCH_SIZE > total_simulations {
                total_simulations - progress
            } else {
                MAX_BATCH_SIZE
            };

            progress += batch_size;

            let strategy = strategy.clone();
            let decklist = decklist.clone();
            let scope = scope.clone();

            spawn_local(async move {
                let results = Goldfish::run_simulations(&strategy, &decklist, batch_size);
                if progress == total_simulations {
                    scope.respond(
                        id,
                        Status::Complete(total_simulations, results),
                    );
                } else {
                    scope.respond(
                        id,
                        Status::InProgress(progress, total_simulations, results),
                    );
                }
            });
        }
    }
}
