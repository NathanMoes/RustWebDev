use yew::prelude::*;

/// A function component for the footer of the application
#[function_component(Footer)]
pub fn footer() -> Html {
    html! {
        <footer>
            <p>{ "Nathan Moes's Question app © 2024" }</p>
        </footer>
    }
}
