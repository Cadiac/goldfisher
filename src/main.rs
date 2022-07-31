use log::debug;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlSelectElement, HtmlInputElement};
use yew::prelude::*;

use goldfisher::deck::Decklist;
use goldfisher::game::{Game, GameResult};
use goldfisher::strategy::{aluren, pattern_hulk, Strategy};

mod components;

#[derive(Debug)]
pub enum Msg {
    SelectStrategy(String),
    ChangeSimulationsCount(usize),
    BeginSimulation,
    FinishSimulation,
}

pub struct App {
    strategies: Vec<Rc<Box<dyn Strategy>>>,
    current_strategy: Option<Rc<Box<dyn Strategy>>>,
    decklist: Option<Decklist>,
    simulations: usize,
    statistics: HashMap<(GameResult, usize), usize>,
    result: Option<String>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let strategies: Vec<Rc<Box<dyn Strategy>>> = vec![
            Rc::new(Box::new(pattern_hulk::PatternHulk {})),
            Rc::new(Box::new(aluren::Aluren {})),
        ];

        Self {
            strategies,
            current_strategy: None,
            decklist: None,
            simulations: 100,
            statistics: HashMap::new(),
            result: None,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}

    fn destroy(&mut self, _: &Context<Self>) {}

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        debug!("[Update]: {msg:?}");

        match msg {
            Msg::SelectStrategy(name) => {
                if let Some(strategy) = self
                    .strategies
                    .iter()
                    .find(|strategy| strategy.name() == name)
                {
                    self.decklist = Some(strategy.default_decklist());
                    self.current_strategy = Some(strategy.clone());
                }
            }
            Msg::ChangeSimulationsCount(count) => {
                self.simulations = count;
            }
            Msg::BeginSimulation => {
                for _ in 0..self.simulations {
                    let mut game = Game::new(self.decklist.as_ref().unwrap());
                    let (result, turn) = game.run(&self.current_strategy.as_ref().unwrap());
                    *self.statistics.entry((result, turn)).or_insert(0) += 1;
                }
                let link = ctx.link();

                link.send_message(Msg::FinishSimulation);
            }
            Msg::FinishSimulation => {
                let mut wins_by_turn = self
                    .statistics
                    .iter()
                    .filter(|((result, _), _)| *result == GameResult::Win)
                    .map(|((_, turn), count)| (*turn, *count))
                    .collect::<Vec<_>>();

                let mut losses_by_turn = self
                    .statistics
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
                    / total_wins as f32;

                let mut output = vec![];
                output.push(format!("=======================[ RESULTS ]=========================="));
                output.push(format!("                   Average turn: {average_turn:.2}"));
                output.push(format!("              Wins per turn after {simulations} games:", simulations = self.simulations));
                output.push(format!("============================================================"));

                let mut cumulative = 0.0;
                for (turn, wins) in wins_by_turn {
                    let win_percentage = 100.0 * wins as f32 / self.simulations as f32;
                    cumulative += win_percentage;
                    output.push(format!("Turn {turn:002}: {wins} wins ({win_percentage:.1}%) - cumulative {cumulative:.1}%"));
                }

                let mut loss_cumulative = 0.0;
                for (turn, losses) in losses_by_turn {
                    let loss_percentage = 100.0 * losses as f32 / self.simulations as f32;
                    loss_cumulative += loss_percentage;
                    output.push(format!("Turn {turn:002}: {losses} losses ({loss_percentage:.1}%) - cumulative {loss_cumulative:.1}%"));
                }

                self.result = Some(output.join("\n"));
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        html! {
            <div>
                <h1>{ "Goldfisher" }</h1>

                <div>
                    <label for="strategy-select">{"Choose a deck strategy:"}</label>
                    <select name="strategies" id="strategy-select" onchange={link.batch_callback(move |e: Event| {
                        let target: Option<EventTarget> = e.target();
                        let select = target.and_then(|t| t.dyn_into::<HtmlSelectElement>().ok());
                        select.map(|select| Msg::SelectStrategy(select.value()))
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
                    <textarea id="decklist" name="decklist" rows="30" cols="35" placeholder="Choose deck strategy.." value={
                        match &self.decklist {
                            None => String::new(),
                            Some(decklist) => decklist.to_string()
                        }
                    }/>
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
                    <button type="button" onclick={link.callback(|_| Msg::BeginSimulation)}>{"Start"}</button>
                </div>

                <div>
                    {if let Some(result) = &self.result {
                        html! {
                            <pre>{result}</pre>
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

    let strategy: Box<dyn Strategy> = Box::new(pattern_hulk::PatternHulk {});
    let decklist = strategy.default_decklist();
    let mut game = Game::new(&decklist);
    game.run(&strategy);
}
