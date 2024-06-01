use crate::*;
use gloo_net::http::Request;
use serde::Serialize;
use web_sys::HtmlInputElement;

#[derive(Serialize)]
struct QuestionData {
    title: String,
    content: String,
    tags: Option<String>,
}

#[function_component(QuestionForm)]
pub fn question_form() -> Html {
    let title = use_state(String::new);
    let content = use_state(String::new);
    let tags = use_state(String::new);

    let onsubmit = {
        let title = title.clone();
        let content = content.clone();
        let tags = tags.clone();

        Callback::from(move |e: FocusEvent| {
            e.prevent_default();

            let question_data = QuestionData {
                title: (*title).clone(),
                content: (*content).clone(),
                tags: Some((*tags).clone()),
            };

            wasm_bindgen_futures::spawn_local(async move {
                let request = Request::post("http://localhost:8000/questions")
                    .json(&question_data)
                    .unwrap();

                let response = request.send().await;
                match response {
                    Ok(_) => {
                        // Success, redirect to main page/list page?
                    }
                    Err(err) => {
                        eprintln!("Error submitting question: {}", err);
                    }
                }
            });
        })
    };

    html! {
        <form onsubmit={onsubmit}>
            <div>
                <label for="title">{ "Title:" }</label>
                <input type="text" id="title" oninput={move |e: InputEvent| title.set(e.target_unchecked_into::<HtmlInputElement>().value())} />
            </div>
            <div>
                <label for="content">{ "Content:" }</label>
                <textarea id="content" oninput={move |e: InputEvent| content.set(e.target_unchecked_into::<HtmlInputElement>().value())}></textarea>
            </div>
            <div>
                <label for="tags">{ "Tags (comma-separated):" }</label>
                <input type="text" id="tags" oninput={move |e: InputEvent| tags.set(e.target_unchecked_into::<HtmlInputElement>().value())} />
            </div>
            <button type="submit">{ "Submit" }</button>
        </form>
    }
}
