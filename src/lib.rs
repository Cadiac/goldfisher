use std::rc::Rc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use gloo_worker::{HandlerId, Worker, WorkerScope};

use wasm_bindgen_futures::spawn_local;

use goldfisher::deck::{Decklist};
use goldfisher::game::{Game, GameResult};
use goldfisher::strategy::{Strategy, pattern_hulk, aluren};

#[derive(Debug)]
pub enum Msg<T> {
    Respond { output: T, id: HandlerId },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    InProgress(usize, usize, Results),
    Complete(Results),
}

type Results = HashMap<(GameResult, usize), usize>;

pub struct Goldfish {}

impl Goldfish {
    fn run_simulations(strategy: &Rc<Box<dyn Strategy>>, decklist: &Decklist, count: usize) {
        let mut results = HashMap::new();

        for _ in 0..count {
            let mut game = Game::new(&decklist);
            let (result, turn) = game.run(strategy);
            *results.entry((result, turn)).or_insert(0) += 1;
        }
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

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, who: HandlerId) {
        let (strategy_name, decklist, simulations) = msg;

        let mut results = HashMap::new();
        let strategy: Rc<Box<dyn Strategy>> = match strategy_name.as_str() {
            pattern_hulk::NAME => Rc::new(Box::new(pattern_hulk::PatternHulk {})),
            aluren::NAME => Rc::new(Box::new(aluren::Aluren {})),
            _ => unimplemented!()
        };

        for current_simulation in 0..10 {
            let sc = strategy.clone();
            let dc = decklist.clone();

            spawn_local(async move {
                Goldfish::run_simulations(&sc, &dc, 100);
            });

            scope.send_message(Msg::Respond { output: Status::InProgress(current_simulation, simulations, results.clone()), id: who });
        }

        // for current_simulation in 0..simulations {
        //     let mut game = Game::new(&decklist);
        //     let (result, turn) = game.run(&strategy);
        //     *results.entry((result, turn)).or_insert(0) += 1;

        //     if current_simulation % 100 == 0 {
        //         scope.send_message(Msg::Respond { output: Status::InProgress(current_simulation, simulations, results.clone()), id: who });
        //     }
        // }

        scope.send_message(Msg::Respond { output: Status::Complete(results), id: who });
    }
}
