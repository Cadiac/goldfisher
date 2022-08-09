use gloo_worker::{HandlerId, Worker, WorkerScope};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use js_sys::Promise;
use web_sys::WorkerGlobalScope;

use goldfisher::deck::Decklist;
use goldfisher::game::{Game, GameResult};
use goldfisher::strategy::{aluren, pattern_hulk, Strategy};

const MAX_BATCH_SIZE: usize = 10;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Cmd {
    Begin(String, String, usize),
    Cancel,
}

#[derive(Debug, PartialEq)]
enum State {
    Idle,
    Running,
    Cancelling,
}

#[derive(Debug)]
pub enum Msg {
    Command { cmd: Cmd, id: HandlerId },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    InProgress(usize, usize, Vec<(GameResult, usize)>),
    Complete(usize, Vec<(GameResult, usize)>),
    Error(String),
}

/// Worker timer, which setTimeout is created by WorkerGlobalScope
/// This is necessary because worker has no access to windows.
/// Source: extraymond @ https://extraymond.github.io/posts/2019-08-25-let-s-create-a-task-manager-in-webworker/
pub async fn worker_timer(ms: i32) -> Result<(), JsValue> {
    let promise = Promise::new(&mut |yes, _| {
        let global = js_sys::global();
        let scope = global.dyn_into::<WorkerGlobalScope>().unwrap();
        scope
            .set_timeout_with_callback_and_timeout_and_arguments_0(&yes, ms)
            .unwrap();
    });
    let js_fut = JsFuture::from(promise);
    js_fut.await?;
    Ok(())
}

pub struct Goldfish {
    state: Arc<Mutex<State>>,
}

impl Goldfish {
    async fn run(
        state: Arc<Mutex<State>>,
        scope: WorkerScope<Self>,
        id: HandlerId,
        strategy_name: String,
        decklist_str: String,
        total_simulations: usize,
    ) {
        {
            let mut state = state.lock().unwrap();
            if *state != State::Idle {
                return;
            }

            *state = State::Running;
        }

        let strategy: Rc<Box<dyn Strategy>> = match strategy_name.as_str() {
            pattern_hulk::NAME => Rc::new(Box::new(pattern_hulk::PatternHulk {})),
            aluren::NAME => Rc::new(Box::new(aluren::Aluren {})),
            _ => {
                scope.respond(
                    id,
                    Status::Error(format!("unsupported strategy \"{strategy_name}\"")),
                );
                return;
            }
        };

        let decklist = match decklist_str.parse::<Decklist>() {
            Ok(decklist) => decklist,
            Err(err) => {
                scope.respond(
                    id,
                    Status::Error(format!("failed to parse decklist: {err:?}")),
                );
                return;
            }
        };

        let mut progress = 0;
        scope.respond(
            id,
            Status::InProgress(progress, total_simulations, Vec::new()),
        );

        loop {
            if progress >= total_simulations {
                break;
            }

            {
                let state = state.lock().unwrap();
                if *state == State::Cancelling {
                    break;
                }
            }

            worker_timer(0).await.unwrap();

            let batch_size = if progress + MAX_BATCH_SIZE > total_simulations {
                total_simulations - progress
            } else {
                MAX_BATCH_SIZE
            };

            progress += batch_size;

            match Goldfish::run_batch(&strategy, &decklist, batch_size) {
                Ok(results) => {
                    if progress == total_simulations {
                        scope.respond(id, Status::Complete(total_simulations, results));
                    } else {
                        scope.respond(id, Status::InProgress(progress, total_simulations, results));
                    }
                }
                Err(err) => {
                    scope.respond(
                        id,
                        Status::Error(format!("failed to simulate games: {err:?}")),
                    );
                }
            }
        }

        let mut state = state.lock().unwrap();
        *state = State::Idle;
    }

    fn run_batch(
        strategy: &Rc<Box<dyn Strategy>>,
        decklist: &Decklist,
        batch_size: usize,
    ) -> Result<Vec<(GameResult, usize)>, Box<dyn Error>> {
        let mut results = Vec::new();

        for _ in 0..batch_size {
            let strategy = strategy.clone();
            let mut game = Game::new(&decklist)?;
            let result = game.run(&strategy);
            results.push(result);
        }

        Ok(results)
    }

    fn cancel(&mut self) {
        let mut state = self.state.lock().unwrap();
        *state = State::Cancelling;
    }
}

impl Worker for Goldfish {
    type Input = Cmd;

    type Message = Msg;

    type Output = Status;

    fn create(_scope: &WorkerScope<Self>) -> Self {
        Self {
            state: Arc::new(Mutex::new(State::Idle)),
        }
    }

    fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message) {
        match msg {
            Msg::Command { cmd, id } => {
                match cmd {
                    Cmd::Begin(strategy_name, decklist_str, total_simulations) => {
                        let (state, scope) = (Arc::clone(&self.state), scope.clone());

                        spawn_local(async move {
                            Goldfish::run(
                                state,
                                scope,
                                id,
                                strategy_name,
                                decklist_str,
                                total_simulations,
                            ).await;
                        });
                    }
                    Cmd::Cancel => self.cancel(),
                }
            }
        }
    }

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId) {
        scope.send_message(Msg::Command { cmd: msg, id })
    }
}
