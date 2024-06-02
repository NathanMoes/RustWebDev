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

#[derive(Properties, PartialEq)]
pub struct QuestionFormProps {
    #[prop_or_default]
    pub question_id: Option<u32>,
}

/// An answer struct to represent an answer in the database
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Answer {
    pub content: String,
    pub question_id: u32,
}

/// A function component that displays a list of questions from the server backend. With a start end end parameter, it can also display a single question. By default it will only display one at the moment
#[function_component(QuestionItem)]
pub fn question(&QuestionFormProps { question_id }: &QuestionFormProps) -> Html {
    let question = use_state(|| None);
    let history = use_history().unwrap();
    let answers = use_state(Vec::<Answer>::new);

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
        let question = question.clone();
        let answers = answers.clone();

        use_effect_with_deps(
            move |_| {
                let question = question.clone();
                let answers = answers.clone();
                let id = question_id.unwrap_or_default();

                wasm_bindgen_futures::spawn_local(async move {
                    let request = Request::get(&format!(
                        "http://localhost:8000/questions?start={}&end={}",
                        id, id
                    ))
                    .send()
                    .await;
                    match request {
                        Ok(response) => {
                            let questions_data: Vec<Question> =
                                response.json().await.unwrap_or_default();
                            if let Some(question_data) = questions_data.first() {
                                question.set(Some(question_data.clone()));
                            }

                            let request =
                                Request::get(&format!("http://localhost:8000/answers?id={}", id));
                            let response = request.send().await;
                            match response {
                                Ok(response) => {
                                    let answers_data: Vec<Answer> =
                                        response.json().await.unwrap_or_default();
                                    answers.set(answers_data);
                                }
                                Err(err) => {
                                    eprintln!("Error fetching answers: {}", err);
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("Error fetching question: {}", err);
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
            {
                question.as_ref().map(|question| {
                    let id = question.id;
                    let history = history.clone();
                    let history2 = history.clone();
                    html! {
                        <div class="question">
                            <h2 class="title">{ &question.title }</h2>
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
                                <button onclick={move |_| {
                                    history2.push(Route::Answer{id});
                                }}>{ "Add Answer" }</button>
                            </div>
                        </div>
                    }
                }).unwrap_or_else(|| html! {
                    <p>{ "Question not found" }</p>
                })
            }

            <h3>{ "Answers" }</h3>
            <div class="answer-list">
                {
                    answers.iter().map(|answer| {
                        html! {
                            <div class="answer">
                                <div class="content">{ &answer.content }</div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
        </>
    }
}
