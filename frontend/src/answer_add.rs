use std::collections::HashSet;

use crate::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;

#[derive(Serialize)]
struct QuestionData {
    id: u32,
    title: String,
    content: String,
    tags: Option<HashSet<String>>,
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

/// A function component form for submitting a new question
#[function_component(AnswerAdd)]
pub fn question_form(&QuestionFormProps { question_id }: &QuestionFormProps) -> Html {
    let history = use_history().unwrap();
    let content = use_state(String::new);

    let onsubmit = {
        let content = content.clone();
        let history_clone = history.clone();

        Callback::from(move |e: FocusEvent| {
            e.prevent_default();

            let history_clone_for_async = history_clone.clone();
            let answer_data = Answer {
                content: content.to_string(),
                question_id: question_id.unwrap_or_default(),
            };

            wasm_bindgen_futures::spawn_local(async move {
                let request = Request::post("http://localhost:8000/answers")
                    .json(&answer_data)
                    .unwrap();

                let response = request.send().await;
                match response {
                    Ok(response) => {
                        if response.ok() {
                            // Success, redirect to main page/list page
                            history_clone_for_async.push(Route::Question {
                                id: question_id.unwrap_or_default(),
                            });
                            web_sys::console::log_1(&"Answer submitted successfully".into());
                        } else {
                            let error_message = response
                                .text()
                                .await
                                .unwrap_or_else(|_| "Unknown error".to_string());
                            web_sys::console::error_1(&error_message.into());
                        }
                    }
                    Err(err) => {
                        web_sys::console::error_1(&err.to_string().into());
                    }
                }
            });
        })
    };

    html! {
        <form class="question-form" onsubmit={onsubmit}>
            <div class="form-group">
                <label for="content">{ "Content:" }</label>
                <textarea id="content" class="form-textarea" oninput={move |e: InputEvent| content.set(e.target_unchecked_into::<HtmlInputElement>().value())}></textarea>
            </div>
            <button type="submit" class="submit-button">{ "Submit" }</button>
        </form>
    }
}
