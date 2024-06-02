use yew::prelude::*;

#[function_component(Footer)]
pub fn footer() -> Html {
    html! {
        <footer>
            <p>{ "Nathan Moes's Question app © 2024" }</p>
        </footer>
    }
}
