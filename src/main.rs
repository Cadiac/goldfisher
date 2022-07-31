use std::rc::Rc;
use log::{debug};
use yew::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlSelectElement};

use goldfisher::game::Game;
use goldfisher::strategy::{aluren, pattern_hulk, Strategy};

mod components;

#[derive(Debug)]
pub enum Msg {
    OnSelectStrategy(String),
}

pub struct App {
    strategies: Vec<Rc<Box<dyn Strategy>>>,
    current_strategy: Option<Rc<Box<dyn Strategy>>>,
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
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}

    fn destroy(&mut self, _: &Context<Self>) {}

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        debug!("[Update]: {msg:?}");

        match msg {
            Msg::OnSelectStrategy(name) => {
                if let Some(strategy) = self.strategies.iter().find(|strategy| strategy.name() == name) {
                    self.current_strategy = Some(strategy.clone());
                }
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
                        select.map(|select| Msg::OnSelectStrategy(select.value()))
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
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();

    let strategy: Box<dyn Strategy> = Box::new(pattern_hulk::PatternHulk {});
    let decklist = strategy.default_decklist();
    let mut game = Game::new(&decklist);
    game.run(&strategy);
}
