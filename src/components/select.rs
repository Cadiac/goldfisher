use yew::prelude::*;

use crate::Msg;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub name: String,
    pub on_select: Callback<Msg>,
    pub options: Vec<(String, String, bool)>,
}

#[function_component(Select)]
pub fn select(props: &Props) -> Html {
    // TODO - Placeholder: --Please choose an option--

    html! {
        <div>
            <label for="deck-select">{"Choose a deck:"}</label>
            <select name="decks" id="deck-select">
                {
                    props.options.iter().map(|(value, label, is_selected)| {
                        html! { <option selected={*is_selected} value={value.clone()}>{label}</option> }
                    })
                    .collect::<Html>()
                }
            </select>
        </div>
    }
}
