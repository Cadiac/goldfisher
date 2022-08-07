use gloo_worker::{Spawnable, WorkerBridge};
use log::info;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;

use goldfisher::deck::Decklist;
use goldfisher::game::GameResult;
use goldfisher::strategy::{aluren, pattern_hulk, Strategy};

use goldfisher_web::{Goldfish, Status};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Debug)]
pub enum Msg {
    ChangeStrategy(String),
    ChangeSimulationsCount(usize),
    ChangeDecklist(String),
    BeginSimulation,
    UpdateProgress(usize, usize, Vec<(GameResult, usize)>),
    FinishSimulation(usize, Vec<(GameResult, usize)>),
    SimulationError(String),
}

impl fmt::Display for Msg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Msg::ChangeStrategy(name) => write!(f, "ChangeStrategy(\"{name}\")"),
            Msg::ChangeSimulationsCount(count) => write!(f, "ChangeSimulationsCount({count})"),
            Msg::ChangeDecklist(_decklist) => write!(f, "ChangeDecklist"),
            Msg::BeginSimulation => write!(f, "BeginSimulation"),
            Msg::UpdateProgress(current, total, _results) => {
                write!(f, "UpdateProgress({current}, {total})")
            }
            Msg::FinishSimulation(total, _results) => write!(f, "FinishSimulation({total})"),
            Msg::SimulationError(message) => write!(f, "SimulationError({message:?})"),
        }
    }
}

pub struct App {
    strategies: Vec<Rc<Box<dyn Strategy>>>,
    current_strategy: Option<Rc<Box<dyn Strategy>>>,
    decklist: String,
    is_busy: bool,
    simulations: usize,
    progress: (usize, usize),
    results: HashMap<(GameResult, usize), usize>,
    output: Option<String>,
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
            output.push(format!(
                "                    Finished: {progress}/{total_simulations}",
            ));
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

        self.output = Some(output.join("\n"));
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
                    Status::Complete(total, results) => {
                        link.send_message(Msg::FinishSimulation(total, results))
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
            simulations: 100,
            progress: (0, 0),
            results: HashMap::new(),
            output: None,
            worker,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}

    fn destroy(&mut self, _: &Context<Self>) {}

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        info!("[Update]: {msg}");

        match msg {
            Msg::ChangeStrategy(name) => {
                if let Some(strategy) = self
                    .strategies
                    .iter()
                    .find(|strategy| strategy.name() == name)
                {
                    self.decklist = strategy.default_decklist().to_string();
                    self.current_strategy = Some(strategy.clone());
                }
            }
            Msg::ChangeSimulationsCount(count) => {
                self.simulations = count;
            }
            Msg::ChangeDecklist(decklist_str) => match decklist_str.parse::<Decklist>() {
                Ok(_) => {
                    self.decklist = decklist_str;
                }
                Err(err) => self.output = Some(err.to_string()),
            },
            Msg::BeginSimulation => {
                if !self.decklist.is_empty() {
                    self.is_busy = true;
                    self.results.clear();
                    self.worker.send((
                        pattern_hulk::NAME.to_owned(),
                        self.decklist.clone(),
                        self.simulations,
                    ));
                }
            }
            Msg::UpdateProgress(progress, total_simulations, results) => {
                for result in results {
                    *self.results.entry(result).or_insert(0) += 1;
                }

                self.progress = (progress, total_simulations);
                self.update_output();
            }
            Msg::FinishSimulation(total_simulations, results) => {
                for result in results {
                    *self.results.entry(result).or_insert(0) += 1;
                }

                self.progress = (total_simulations, total_simulations);
                self.is_busy = false;
                self.update_output();
            }
            Msg::SimulationError(message) => {
                self.is_busy = false;
                self.output = Some(message);
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        html! {
            <div>
                <h1>{ "Goldfisher 🎣" }</h1>

                <div>
                    <label for="strategy-select">{"Choose a deck strategy:"}</label>
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
                                        selected={name.clone() == self.current_strategy.as_ref().map(|s| s.name()).unwrap_or(String::from(""))}
                                        value={name.clone()}>
                                        {name}
                                    </option> }
                            })
                            .collect::<Html>()
                        }
                    </select>
                </div>

                <div>
                    <label for="decklist">{"Decklist:"}</label>
                    <textarea id="decklist" name="decklist" rows="30" cols="35" placeholder="Choose deck strategy.." value={self.decklist.clone()}
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

                <div>
                    <label for="simulated-games">{"Games to simulate:"}</label>
                    <input type="number" id="simulated-games" name="tentacles" min="1" max="1000000" value={self.simulations.to_string()}
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

                <div>
                    <label for="run-simulation">{"Run simulation:"}</label>
                    <button type="button"
                        disabled={self.is_busy || self.current_strategy.is_none() || self.decklist.is_empty()}
                        onclick={link.callback(|_| Msg::BeginSimulation)}>
                        { "Run simulation" }
                    </button>
                </div>

                <div>
                    {if let Some(output) = &self.output {
                        html! {
                            <pre>{output}</pre>
                        }
                    } else {
                        html! {}
                    }}
                </div>

            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    yew::start_app::<App>();
}
