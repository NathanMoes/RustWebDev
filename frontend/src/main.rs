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
mod question_update;

use components::footer::Footer;
use components::header::Header;
use question_form::QuestionForm as Form;
use question_list::QuestionList as List;
use question_update::{QuestionFormProps, QuestionUpdate as Update};

#[derive(Clone, Routable, PartialEq, Debug)]
enum Route {
    #[at("/")]
    List,
    #[at("/questions/add")]
    Form,
    #[at("/questions/update/{id}")]
    Update { id: u32 },
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
        <div class="not-found">
            <h1>{ "404 - Page Not Found" }</h1>
            <p style={"text-align: center"}>{ "The requested page could not be found." }</p>
        </div>
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Header />
            <Switch<Route> render={RenderFn::new(move |route: &Route| {
                log::info!("Matched route: {:?}", route);
                match route {
                    Route::List => html! { <List /> },
                    Route::Form => html! { <Form /> },
                    Route::Update { id } => {
                        let props = QuestionFormProps {
                            question_id: Some(*id),
                        };
                        html! { <Update ..props /> }
                    }
                    Route::NotFound => html! { <NotFound /> },
                }
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
