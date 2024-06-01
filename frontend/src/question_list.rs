use std::collections::HashSet;

use crate::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use web_sys::window;

#[derive(Deserialize, Clone, PartialEq, Serialize)]
pub struct Question {
    pub id: u32,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub tags: Option<HashSet<String>>,
}

#[function_component(QuestionList)]
pub fn question_form() -> Html {
    let questions = use_state(Vec::<Question>::new);

    fn handle_delete_question(id: u32) {
        wasm_bindgen_futures::spawn_local(async move {
            let request = Request::delete(&format!("http://localhost:8000/questions?id={}", id))
                .send()
                .await;
            match request {
                Ok(response) => {
                    if response.ok() {
                        // Success, refresh the list of questions
                        window().unwrap().location().reload().unwrap();
                    }
                }
                Err(err) => {
                    eprintln!("Error deleting question: {}", err);
                }
            }
        });
    }

    {
        let questions = questions.clone();

        use_effect_with_deps(
            move |_| {
                let questions = questions.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let request = Request::get("http://localhost:8000/questions").send().await;
                    match request {
                        Ok(response) => {
                            let questions_data: Vec<Question> =
                                response.json().await.unwrap_or_default();
                            questions.set(questions_data);
                        }
                        Err(err) => {
                            eprintln!("Error fetching questions: {}", err);
                        }
                    }
                });

                || {}
            },
            (),
        );
    }

    html! {
        <div>
            <h1>{ "Questions" }</h1>
            <pre>{
                questions.iter().map(|question| {
                    let id = question.id;
                    html! {
                        <div class="question">
                            <div class="id">{ question.id }</div>
                            <div class="title">{ &question.title }</div>
                            <div class="content">{ &question.content }</div>
                            <div class="tags">{
                                question.tags.as_ref().map(|tags| {
                                    tags.iter().map(|tag| {
                                        html! { <span class="tag">{ tag }</span> }
                                    }).collect::<Html>()
                                }).unwrap_or_else(|| html! {})
                            }</div>
                            <button>{ "Edit" }</button>
                            <button onclick={move |_| {
                                handle_delete_question(id);
                            }}>{ "Delete" }</button>
                        </div>
                    }
                }).collect::<Html>()
            }</pre>
        </div>
    }
}
