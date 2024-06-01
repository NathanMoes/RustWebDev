use crate::api::{ApiError, IdParam};
use crate::*;

/// Web function to get a single question from the questions
pub async fn get_question(
    State(state): State<AppState>,
    Query(IdParam { id }): Query<IdParam>,
) -> impl IntoResponse {
    match id {
        Some(id) => {
            let question_id = QuestionId(id);
            match state.get_question(&question_id).await {
                Ok(question) => Response::builder()
                    .status(StatusCode::OK)
                    .body(serde_json::to_string_pretty(&question).unwrap())
                    .unwrap(),
                Err(_) => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(ApiError::QuestionNotFound.to_string())
                    .unwrap(),
            }
        }
        None => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(ApiError::MissingParameters.to_string())
            .unwrap(),
    }
}

/// Entry point for the web server
pub async fn get_entry_point() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .body("Welcome to the questions and answers service by Nathan Moes!".to_string())
        .unwrap()
}
