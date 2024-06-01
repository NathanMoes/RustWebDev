// use web_sys::{HtmlInputElement, SubmitEvent};
use yew::prelude::*;
use yew_router::{prelude::*, RenderFn};

mod question_form;
mod question_list;

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
    let render = RenderFn::new(move |route: &Route| match route {
        Route::QuestionList => {
            html! { <QuestionList /> }
        }
        Route::QuestionForm => {
            html! { <QuestionForm /> }
        }
    });

    html! {
        <BrowserRouter>
            <Switch<Route> render={render} />
        </BrowserRouter>
    }
}

fn main() {
    yew::start_app::<App>();
}
