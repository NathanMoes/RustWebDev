// use web_sys::{HtmlInputElement, SubmitEvent};
#![allow(clippy::let_unit_value)]
use console_error_panic_hook::set_once as set_panic_hook;
use console_log::init_with_level;
use log::Level;
use yew::prelude::*;
use yew_router::{prelude::*, RenderFn};
mod components;
mod question_form;
mod question_list;

use components::footer::Footer;
use components::header::Header;
use question_form::QuestionForm;
use question_list::QuestionList;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    QuestionList,
    #[at("/questions/new")]
    QuestionForm,
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Header />
            <Switch<Route> render={RenderFn::new(move |route: &Route| match route {
                Route::QuestionList => html! { <QuestionList /> },
                Route::QuestionForm => html! { <QuestionForm /> },
            })} />
            <Footer />
        </BrowserRouter>
    }
}
fn main() {
    set_panic_hook();
    init_with_level(Level::Info).expect("Failed to initialize logger");
    yew::start_app::<App>();
}
