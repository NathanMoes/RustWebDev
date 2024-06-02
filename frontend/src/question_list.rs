use std::collections::HashSet;

use crate::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::window;

#[derive(Deserialize, Clone, PartialEq, Serialize)]
pub struct Question {
    pub id: u32,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub tags: Option<HashSet<String>>,
}

/// A function component that displays a list of questions from the server backend
#[function_component(QuestionList)]
pub fn question_form() -> Html {
    let questions = use_state(Vec::<Question>::new);
    let history = use_history().unwrap();

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
        <>
            <h1>{ "Questions" }</h1>
            <div class="question-list">
                {
                    questions.iter().map(|question| {
                        let id = question.id;
                        let history = history.clone();
                        let item_history = history.clone();
                        html! {
                            <div class="question" onclick={move |_|{
                                item_history.push(Route::Question{id})
                            }}>
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
                                <div class="actions">
                                    <button onclick={move |_|{
                                        history.push(Route::Update{id});
                                    }}>{ "Edit" }</button>
                                    <button onclick={move |_| {
                                        handle_delete_question(id);
                                    }}>{ "Delete" }</button>
                                </div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
        </>
    }
}
