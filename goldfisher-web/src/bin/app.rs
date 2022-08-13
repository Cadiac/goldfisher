use gloo_worker::{Spawnable, WorkerBridge};
use log::debug;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;

use goldfisher::deck::Deck;
use goldfisher::game::GameResult;
use goldfisher::strategy::{aluren, pattern_hulk, Strategy};

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
    UpdateProgress(usize, usize, Vec<(GameResult, usize)>),
    FinishSimulation(usize, usize, Vec<(GameResult, usize)>),
    SimulationError(String),
}

impl fmt::Display for Msg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Msg::ChangeStrategy(name) => write!(f, "ChangeStrategy(\"{name}\")"),
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
        }
    }
}

pub struct App {
    strategies: Vec<Rc<Box<dyn Strategy>>>,
    current_strategy: Option<Rc<Box<dyn Strategy>>>,
    decklist: String,
    is_busy: bool,
    is_decklist_error: bool,
    simulations: usize,
    progress: (usize, usize),
    results: HashMap<(GameResult, usize), usize>,
    output: String,
    worker: WorkerBridge<Goldfish>,
}

impl App {
    fn update_output(&mut self) {
        let progress: usize = self.progress.0;
        let total_simulations = self.progress.1;

        let mut wins_by_turn = self
            .results
            .iter()
            .filter(|((result, _), _)| *result == GameResult::Win)
            .map(|((_, turn), count)| (*turn, *count))
            .collect::<Vec<_>>();

        let mut losses_by_turn = self
            .results
            .iter()
            .filter(|((result, _), _)| *result == GameResult::Lose)
            .map(|((_, turn), count)| (*turn, *count))
            .collect::<Vec<_>>();

        wins_by_turn.sort();
        losses_by_turn.sort();

        let total_wins: usize = wins_by_turn.iter().map(|(_, wins)| *wins).sum();
        let average_turn = wins_by_turn
            .iter()
            .map(|(turn, wins)| *turn * *wins)
            .sum::<usize>() as f32
            / usize::max(total_wins, 1) as f32;

        let mut output = vec![];
        if self.is_busy {
            output.push(format!(
                "=======================[ RUNNING ]=========================="
            ));
            output.push(format!(
                "                   In progress: {progress}/{total_simulations}",
            ));
        } else {
            output.push(format!(
                "=======================[ RESULTS ]=========================="
            ));
            if self.progress.0 == self.progress.1 {
                output.push(format!(
                    "                    Finished: {progress}/{total_simulations}",
                ));
            } else {
                output.push(format!(
                    "                   Cancelled: {progress}/{total_simulations}",
                ));
            }
        }
        output.push(format!(
            "                    Average turn: {average_turn:.2}"
        ));
        output.push(format!(
            "               Wins per turn after {progress} games:"
        ));
        output.push(format!(
            "============================================================"
        ));

        let mut cumulative = 0.0;
        for (turn, wins) in wins_by_turn {
            let win_percentage = 100.0 * wins as f32 / progress as f32;
            cumulative += win_percentage;
            output.push(format!(
                "Turn {turn:002}: {wins} wins ({win_percentage:.1}%) - cumulative {cumulative:.1}%"
            ));
        }

        let mut loss_cumulative = 0.0;
        for (turn, losses) in losses_by_turn {
            let loss_percentage = 100.0 * losses as f32 / progress as f32;
            loss_cumulative += loss_percentage;
            output.push(format!("Turn {turn:002}: {losses} losses ({loss_percentage:.1}%) - cumulative {loss_cumulative:.1}%"));
        }

        self.output = output.join("\n");
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let strategies: Vec<Rc<Box<dyn Strategy>>> = vec![
            Rc::new(Box::new(pattern_hulk::PatternHulk {})),
            Rc::new(Box::new(aluren::Aluren {})),
        ];

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
            strategies,
            current_strategy: None,
            decklist: String::new(),
            is_busy: false,
            is_decklist_error: false,
            simulations: 10000,
            progress: (0, 0),
            results: HashMap::new(),
            output: String::from("========================[ READY ]==========================="),
            worker,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}

    fn destroy(&mut self, _: &Context<Self>) {}

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        debug!("[Update]: {msg}");

        match msg {
            Msg::ChangeStrategy(name) => {
                if let Some(strategy) = self
                    .strategies
                    .iter()
                    .find(|strategy| strategy.name() == name)
                {
                    self.decklist = strategy.default_decklist().to_string();
                    self.current_strategy = Some(strategy.clone());
                } else {
                    self.current_strategy = None;
                }
            }
            Msg::ChangeSimulationsCount(count) => {
                self.simulations = count;
            }
            Msg::ChangeDecklist(decklist_str) => {
                if let Err(err) = decklist_str.parse::<Deck>() {
                    self.is_decklist_error = true;
                    self.output = format!("========================[ ERROR ]===========================\n{}", err)
                } else {
                    self.is_decklist_error = false;
                    self.output = String::from("========================[ READY ]===========================")
                }

                self.decklist = decklist_str;
            },
            Msg::BeginSimulation => {
                if !self.decklist.is_empty() && self.current_strategy.is_some() {
                    self.is_busy = true;
                    self.results.clear();
                    self.worker.send(Cmd::Begin {
                        strategy: self.current_strategy.as_ref().unwrap().name(),
                        decklist: self.decklist.clone(),
                        simulations: self.simulations,
                    });
                }
            }
            Msg::CancelSimulation => {
                self.worker.send(Cmd::Cancel);
            }
            Msg::UpdateProgress(progress, total_simulations, results) => {
                for result in results {
                    *self.results.entry(result).or_insert(0) += 1;
                }

                self.progress = (progress, total_simulations);
                self.update_output();
            }
            Msg::FinishSimulation(progress, total_simulations, results) => {
                for result in results {
                    *self.results.entry(result).or_insert(0) += 1;
                }

                self.progress = (progress, total_simulations);
                self.is_busy = false;
                self.update_output();
            }
            Msg::SimulationError(message) => {
                self.is_busy = false;
                self.output = format!("========================[ ERROR ]===========================\n{}", message)
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        let is_ready =
            !self.is_busy && self.current_strategy.is_some() && !self.decklist.is_empty();

        html! {
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
                                                self.strategies.iter().map(|strategy| {
                                                    let name = strategy.name();

                                                    html! {
                                                        <option
                                                            selected={name == self.current_strategy.as_ref().map(|s| s.name()).unwrap_or(String::from(""))}
                                                            value={name.clone()}>
                                                            {name}
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
                            <div class="box">
                                <progress class="progress is-primary" value={self.progress.0.to_string()} max={self.progress.1.to_string()}>
                                    { format!("{}/{}", self.progress.0, self.progress.1) }
                                </progress>
                            </div>
                            {if !self.output.is_empty() {
                                html! {
                                    <div class="box">
                                        <pre>{&self.output}</pre>
                                    </div>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                    </div>
                </div>
            </section>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    yew::start_app::<App>();
}
