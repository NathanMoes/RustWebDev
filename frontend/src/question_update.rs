use std::collections::HashSet;

use crate::*;
use gloo_net::http::Request;
use serde::Serialize;
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

/// A function component form for updating a question
#[function_component(QuestionUpdate)]
pub fn question_update(&QuestionFormProps { question_id }: &QuestionFormProps) -> Html {
    let history = use_history().unwrap();
    let title = use_state(String::new);
    let content = use_state(String::new);
    let tags = use_state(String::new);

    let onsubmit = {
        let title = title.clone();
        let content = content.clone();
        let tags = tags.clone();
        let history_clone = history.clone();

        Callback::from(move |e: FocusEvent| {
            e.prevent_default();

            let tags_set = tags
                .split(',')
                .map(|tag| tag.trim().to_string())
                .collect::<HashSet<String>>();

            let question_data = QuestionData {
                id: question_id.unwrap_or(0),
                title: (*title).clone(),
                content: (*content).clone(),
                tags: if tags_set.is_empty() {
                    None
                } else {
                    Some(tags_set.iter().cloned().collect::<HashSet<String>>())
                },
            };

            let history_clone_for_async = history_clone.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let url = if let Some(id) = question_id {
                    format!("http://localhost:8000/questions?id={}", id)
                } else {
                    "http://localhost:8000/questions".to_string()
                };

                let request = if question_id.is_some() {
                    Request::put(&url)
                } else {
                    Request::post(&url)
                }
                .json(&question_data)
                .unwrap();

                let response = request.send().await;
                match response {
                    Ok(response) => {
                        if response.ok() {
                            // Success, redirect to main page/list page
                            history_clone_for_async.push(Route::List);
                            web_sys::console::log_1(&"Question submitted successfully".into());
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
                <label for="title">{ "Title:" }</label>
                <input type="text" id="title" class="form-input" oninput={move |e: InputEvent| title.set(e.target_unchecked_into::<HtmlInputElement>().value())} />
            </div>
            <div class="form-group">
                <label for="content">{ "Content:" }</label>
                <textarea id="content" class="form-textarea" oninput={move |e: InputEvent| content.set(e.target_unchecked_into::<HtmlInputElement>().value())}></textarea>
            </div>
            <div class="form-group">
                <label for="tags">{ "Tags (comma-separated):" }</label>
                <input type="text" id="tags" class="form-input" oninput={move |e: InputEvent| tags.set(e.target_unchecked_into::<HtmlInputElement>().value())} />
            </div>
            <button type="submit" class="submit-button">{ if question_id.is_some() { "Update" } else { "Submit" } }</button>
        </form>
    }
}
