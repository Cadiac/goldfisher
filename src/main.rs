use yew::prelude::*;

use goldfisher::game::Game;
use goldfisher::strategy::{Strategy, pattern_hulk};

mod components;

use crate::components::{
    select::Select
};

pub enum Msg {
    OnSelectStrategy(String),
}

pub struct App {
    
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {}

    fn destroy(&mut self, _: &Context<Self>) {}

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            _ => unimplemented!()
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        // on_toggle_help_cb={link.callback(|_| Msg::ToggleHelp)}

        html! {
            <div>
                <h1>{ "Hello World" }</h1>
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();

    let strategy: Box<dyn Strategy> = Box::new(pattern_hulk::PatternHulk{});
    let decklist = strategy.default_decklist();
    let mut game = Game::new(&decklist);
    game.run(&strategy);
}