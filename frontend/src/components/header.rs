use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <header>
            <nav>
                <ul>
                    <li><Link<Route> to={Route::List}>{ "Question List" }</Link<Route>></li>
                    <li><Link<Route> to={Route::Form}>{ "New Question" }</Link<Route>></li>
                </ul>
            </nav>
        </header>
    }
}
