use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <header>
            <nav>
                <ul>
                    <li><Link<Route> to={Route::QuestionList}>{ "Question List" }</Link<Route>></li>
                    <li><Link<Route> to={Route::QuestionForm}>{ "New Question" }</Link<Route>></li>
                </ul>
            </nav>
        </header>
    }
}
