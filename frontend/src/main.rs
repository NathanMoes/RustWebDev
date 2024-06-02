// use web_sys::{HtmlInputElement, SubmitEvent};
#![allow(clippy::let_unit_value)]
use console_error_panic_hook::set_once as set_panic_hook;
use console_log::init_with_level;
use log::Level;
use yew::prelude::*;
use yew_router::{prelude::*, RenderFn};
mod answer_add;
mod components;
mod question;
mod question_form;
mod question_list;
mod question_update;

use answer_add::AnswerAdd;
use components::footer::Footer;
use components::header::Header;
use question::QuestionItem;
use question_form::QuestionForm as Form;
use question_list::QuestionList as List;
use question_update::{QuestionFormProps, QuestionUpdate as Update};

/// The routes for the application
#[derive(Clone, Routable, PartialEq, Debug, Copy)]
enum Route {
    #[at("/")]
    List,
    #[at("/questions/add")]
    Form,
    #[at("/question/:id")]
    Question { id: u32 },
    #[at("/questions/update/:id")]
    Update { id: u32 },
    #[at("/answer/:id")]
    Answer { id: u32 },
    #[not_found]
    #[at("/404")]
    NotFound,
}

/// A component that displays a 404 page
#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
        <div class="not-found">
            <h1>{ "404 - Page Not Found" }</h1>
            <p style={"text-align: center"}>{ "The requested page could not be found." }</p>
        </div>
    }
}

/// The main application component
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
                    Route::Question { id } => html! { <QuestionItem question_id={*id} /> },
                    Route::Answer { id } => {
                        let props = answer_add::QuestionFormProps {
                            question_id: Some(*id),
                        };
                        html! { <AnswerAdd ..props /> }
                    }
                    Route::NotFound => html! { <NotFound /> },
                }
            })} />
            <Footer />
        </BrowserRouter>
    }
}

/// The main function
fn main() {
    set_panic_hook();
    init_with_level(Level::Info).expect("Failed to initialize logger");
    yew::start_app::<App>();
}
