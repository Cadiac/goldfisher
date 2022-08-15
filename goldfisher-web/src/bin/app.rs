use gloo_worker::{Spawnable, WorkerBridge};
use log::debug;
use std::collections::BTreeMap;
use std::fmt;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;

use goldfisher::deck::Deck;
use goldfisher::game::GameResult;
use goldfisher::strategy::{DeckStrategy, STRATEGIES};

use goldfisher_web::{Cmd, Goldfish, Status};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Debug)]
pub enum Msg {
    ChangeStrategy(String),
    ChangeSimulationsCount(usize),
    ChangeDecklist(String),
    BeginSimulation,
    CancelSimulation,
    UpdateProgress(usize, usize, Vec<(GameResult, usize, usize)>),
    FinishSimulation(usize, usize, Vec<(GameResult, usize, usize)>),
    SimulationError(String),
    DismissError,
}

impl fmt::Display for Msg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Msg::ChangeStrategy(name) => write!(f, "ChangeStrategy(\"{name:?}\")"),
            Msg::ChangeSimulationsCount(count) => write!(f, "ChangeSimulationsCount({count})"),
            Msg::ChangeDecklist(_decklist) => write!(f, "ChangeDecklist"),
            Msg::BeginSimulation => write!(f, "BeginSimulation"),
            Msg::CancelSimulation => write!(f, "CancelSimulation"),
            Msg::UpdateProgress(current, total, _results) => {
                write!(f, "UpdateProgress({current}, {total})")
            }
            Msg::FinishSimulation(current, total, _results) => {
                write!(f, "FinishSimulation({current}, {total})")
            }
            Msg::SimulationError(message) => write!(f, "SimulationError({message:?})"),
            Msg::DismissError => write!(f, "DismissError"),
        }
    }
}

#[derive(Debug, Default)]
struct Results {
    wins: BTreeMap<usize, usize>,
    losses: usize,
    average_turn: f32,
    mulligans: Vec<usize>,
    average_mulligans: f32,
    percentage_wins: BTreeMap<usize, f32>,
    cumulative_wins: BTreeMap<usize, f32>,
}

pub struct App {
    current_strategy: Option<DeckStrategy>,
    decklist: String,
    is_busy: bool,
    is_decklist_error: bool,
    error_msg: Option<String>,
    simulations: usize,
    progress: (usize, usize),
    results: Results,
    worker: WorkerBridge<Goldfish>,
}

impl App {
    fn update_results(&mut self, new_results: Vec<(GameResult, usize, usize)>) {
        for (result, turn, mulligan_count) in new_results.into_iter() {
            match result {
                GameResult::Win => {
                    *self.results.wins.entry(turn).or_insert(0) += 1;
                }
                GameResult::Lose | GameResult::Draw => {
                    self.results.losses += 1;
                }
            }
            self.results.mulligans.push(mulligan_count);
        }

        let total_wins: usize = self.results.wins.iter().map(|(_, wins)| *wins).sum();

        self.results.average_turn = self
            .results
            .wins
            .iter()
            .map(|(turn, wins)| *turn * *wins)
            .sum::<usize>() as f32
            / usize::max(total_wins, 1) as f32;

        self.results.average_mulligans = self.results.mulligans.iter().sum::<usize>() as f32
            / usize::max(self.results.mulligans.len(), 1) as f32;

        let progress: usize = self.progress.0;
        let mut cumulative = 0.0;
        for (turn, wins) in self.results.wins.iter() {
            let win_percentage = 100.0 * *wins as f32 / progress as f32;
            cumulative += win_percentage;
            *self.results.percentage_wins.entry(*turn).or_insert(0.0) = win_percentage;
            *self.results.cumulative_wins.entry(*turn).or_insert(0.0) = cumulative;
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();

        let worker = Goldfish::spawner()
            .callback(move |results| {
                match results {
                    Status::InProgress(current, total, results) => {
                        link.send_message(Msg::UpdateProgress(current, total, results))
                    }
                    Status::Cancelled(current, total) => {
                        link.send_message(Msg::FinishSimulation(current, total, Vec::new()))
                    }
                    Status::Complete(total, results) => {
                        link.send_message(Msg::FinishSimulation(total, total, results))
                    }
                    Status::Error(message) => link.send_message(Msg::SimulationError(message)),
                };
            })
            .spawn("/worker.js");

        Self {
            current_strategy: None,
            decklist: String::new(),
            is_busy: false,
            is_decklist_error: false,
            simulations: 10000,
            progress: (0, 0),
            results: Results::default(),
            error_msg: None,
            worker,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}

    fn destroy(&mut self, _: &Context<Self>) {}

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        debug!("[Update]: {msg}");

        match msg {
            Msg::ChangeStrategy(deck_strategy) => match deck_strategy.parse::<DeckStrategy>() {
                Err(_) => {
                    self.current_strategy = None;
                }
                Ok(strategy) => {
                    self.decklist = goldfisher::strategy::from_enum(&strategy)
                        .default_decklist()
                        .to_string();
                    self.current_strategy = Some(strategy);
                }
            },
            Msg::ChangeSimulationsCount(count) => {
                self.simulations = count;
            }
            Msg::ChangeDecklist(decklist_str) => {
                if let Err(err) = decklist_str.parse::<Deck>() {
                    self.is_decklist_error = true;
                    self.error_msg = Some(err.to_string());
                } else {
                    self.is_decklist_error = false;
                    self.error_msg = None;
                }

                self.decklist = decklist_str;
            }
            Msg::BeginSimulation => {
                if !self.decklist.is_empty() && self.current_strategy.is_some() {
                    self.is_busy = true;
                    self.error_msg = None;
                    self.results = Results::default();

                    self.worker.send(Cmd::Begin {
                        strategy: self.current_strategy.as_ref().unwrap().clone(),
                        decklist: self.decklist.clone(),
                        simulations: self.simulations,
                    });
                }
            }
            Msg::CancelSimulation => {
                self.worker.send(Cmd::Cancel);
            }
            Msg::UpdateProgress(progress, total_simulations, results) => {
                self.progress = (progress, total_simulations);
                self.update_results(results);
            }
            Msg::FinishSimulation(progress, total_simulations, results) => {
                self.progress = (progress, total_simulations);
                self.is_busy = false;
                self.update_results(results);
            }
            Msg::SimulationError(message) => {
                self.is_busy = false;
                self.error_msg = Some(message);
            }
            Msg::DismissError => self.error_msg = None,
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        let is_ready = !self.is_busy
            && self.current_strategy.is_some()
            && !self.decklist.is_empty()
            && !self.is_decklist_error;

        let (progress, total_games) = self.progress;

        html! {
            <>
                <section class="section">
                    <div class="container">
                        <h1 class="title">{ "Goldfisher 🎣" }</h1>

                        <div class="columns">
                            <div class="column">
                                <div class="box">
                                    <div class="field">
                                        <label class="label" for="strategy-select">{"Choose a deck strategy:"}</label>
                                        <div class="select is-info">
                                            <select name="strategies" id="strategy-select" onchange={link.batch_callback(move |e: Event| {
                                                let target: Option<EventTarget> = e.target();
                                                let select = target.and_then(|t| t.dyn_into::<HtmlSelectElement>().ok());
                                                select.map(|select| Msg::ChangeStrategy(select.value()))
                                            })}>
                                                <option selected={self.current_strategy.is_none()} value={""}>{"-- Please select a strategy --"}</option>
                                                {
                                                    STRATEGIES.iter().map(|strategy| {
                                                        html! {
                                                            <option
                                                                selected={self.current_strategy.as_ref().map(|current| current == strategy).unwrap_or(false)}
                                                                value={strategy.to_string()}>
                                                                {strategy.to_string()}
                                                            </option> }
                                                    })
                                                    .collect::<Html>()
                                                }
                                            </select>
                                        </div>
                                    </div>

                                    <div class="field">
                                        <label class="label" for="decklist">{"Decklist:"}</label>
                                        <textarea class={if self.is_decklist_error { "textarea is-danger" } else { "textarea is-info"}}
                                            id="decklist"
                                            name="decklist"
                                            rows="15"
                                            placeholder="Choose deck strategy.."
                                            value={self.decklist.clone()}
                                            onchange={link.batch_callback(move |e: Event| {
                                                let target: Option<EventTarget> = e.target();
                                                let textarea = target.and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok());
                                                textarea.map(|textarea| {
                                                    let decklist = textarea.value();
                                                    Msg::ChangeDecklist(decklist)
                                                })
                                            })}
                                        />
                                    </div>

                                    <div class="field">
                                        <label class="label" for="simulated-games">{"Games to simulate:"}</label>
                                        <input class="input is-info" type="number" id="simulated-games" name="tentacles" min="1" max="1000000" value={self.simulations.to_string()}
                                            onchange={link.batch_callback(move |e: Event| {
                                                let target: Option<EventTarget> = e.target();
                                                let select = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
                                                select.map(|select| {
                                                    let count = select.value();
                                                    Msg::ChangeSimulationsCount(count.parse().unwrap_or(100))
                                                })
                                            })}
                                        />
                                    </div>
                                </div>

                                <div class="buttons">
                                    <button class={if is_ready { "button is-primary" } else { "button is-primary is-outlined" }} type="button"
                                        disabled={!is_ready}
                                        onclick={link.callback(|_| Msg::BeginSimulation)}>
                                        <span>{ "Run simulation ▶︎" }</span>
                                    </button>

                                    <button class="button" type="button" disabled={!self.is_busy} onclick={link.callback(|_| Msg::CancelSimulation)}>
                                        { "Cancel" }
                                    </button>
                                </div>
                            </div>

                            <div class="column">
                                {if let Some(message) = self.error_msg.as_ref() {
                                    html! {
                                        <article class="message is-danger">
                                            <div class="message-header">
                                                <p>{"Error:"}</p>
                                                <button
                                                    class="delete"
                                                    aria-label="delete"
                                                    onclick={link.callback(|_| Msg::DismissError)}
                                                />
                                            </div>
                                            <div class="message-body">
                                                {message}
                                            </div>
                                        </article>
                                    }
                                } else {
                                    html! {}
                                }}

                                <div class="box">
                                    <div class="field">
                                        <label class="label">{"Progress:"}</label>
                                        <span class="is-small">{format!("{progress}/{total_games}")}</span>
                                        <progress class="progress is-primary" value={progress.to_string()} max={total_games.to_string()}>
                                            { format!("{progress}/{total_games}") }
                                        </progress>
                                    </div>

                                    <div class="columns">
                                        <div class="column">
                                            <label class="label">{"Average turn:"}</label>
                                            <span class="is-small">{format!("{:.2}", self.results.average_turn)}</span>
                                        </div>
                                        <div class="column">
                                            <label class="label">{"Bricked games:"}</label>
                                            <span class="is-small">{format!("{:.2}", self.results.losses)}</span>
                                        </div>
                                        <div class="column">
                                            <label class="label">{"Average mulligans:"}</label>
                                            <span class="is-small">{format!("{:.2}", self.results.average_mulligans)}</span>
                                        </div>
                                    </div>
                                </div>

                                <div class="box">
                                    <div class="table-container">
                                        <table class="table is-fullwidth is-small">
                                            <thead>
                                                <tr>
                                                    <th>{"Turn"}</th>
                                                    <th>{"Wins"}</th>
                                                    <th>{"Wins (%)"}</th>
                                                    <th>{"Cumulative (%)"}</th>
                                                    <th>{""}</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {if self.results.wins.is_empty() && self.results.losses == 0 {
                                                    html! {
                                                        <tr>
                                                            <th>{"--"}</th>
                                                            <td>{"--"}</td>
                                                            <td>{"--"}</td>
                                                            <td>{"--"}</td>
                                                            <td>{"--"}</td>
                                                        </tr>
                                                    }
                                                } else {
                                                    html! {}
                                                }}
                                                {
                                                    self.results.wins.iter().map(|(turn, wins)| {
                                                        let win_percentage = self.results.percentage_wins.get(turn).unwrap_or(&0.0);
                                                        let cumulative = self.results.cumulative_wins.get(turn).unwrap_or(&0.0);
                                                        html! {
                                                            <tr>
                                                                <th>{turn}</th>
                                                                <td>{wins}</td>
                                                                <td>{format!("{win_percentage:.1}%")}</td>
                                                                <td>{format!("{cumulative:.1}%")}</td>
                                                                <td style="vertical-align: middle">
                                                                    <progress class="progress is-small is-primary" style="min-width: 250px" value={wins.to_string()} max={progress.to_string()} />
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect::<Html>()
                                                }
                                            </tbody>
                                        </table>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>
                <footer class="footer">
                    <div class="content has-text-centered">
                        <p>
                            <strong>{"Goldfisher"}</strong>
                            {" by "}
                            <a href="https://github.com/Cadiac">{"Jaakko Husso"}</a>
                            {". The source code of this tool is "}
                            <a href="https://github.com/Cadiac/goldfisher/blob/master/goldfisher-web/LICENSE">{"MIT"}</a>
                            {" licensed, and can be found from "}
                            <a href="https://github.com/Cadiac/goldfisher">{"here"}</a>
                            {"."}
                        </p>
                    </div>
                </footer>
            </>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    yew::start_app::<App>();
}
