use tracing::{info, instrument};

use crate::database::*;
use crate::*;

/// A pagination struct
///
/// This struct is used to paginate the questions in the API from a start to an end index
/// #Example:
/// ```
///
/// {
///   "start": "1",
///   "end": "5"
/// }
#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    start: Option<QuestionId>,
    end: Option<QuestionId>,
}

/// API function to get all questions or a range of questions from the questions hashmap
///
#[instrument]
pub async fn get_questions(
    State(state): State<AppState>,
    Query(Pagination { start, end }): Query<Pagination>,
) -> impl IntoResponse {
    if start.is_none() && end.is_none() {
        info!("Getting all questions");
        let questions = state.questions.read().await;
        Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string_pretty(&questions.clone()).unwrap())
            .unwrap()
    } else {
        let questions = state.questions.read().await;
        let mut result = HashMap::new();
        let start_index;
        let end_index;
        match start {
            Some(s) => match s.0.parse::<usize>().map_err(ApiError::ParseError) {
                Ok(index) => start_index = index,
                Err(e) => {
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(e.to_string())
                        .unwrap();
                }
            },
            None => {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(ApiError::MissingParameters.to_string())
                    .unwrap();
            }
        }
        match end {
            Some(_) => {
                match end
                    .unwrap()
                    .0
                    .parse::<usize>()
                    .map_err(ApiError::ParseError)
                {
                    Ok(index) => end_index = index,
                    Err(e) => {
                        return Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(e.to_string())
                            .unwrap();
                    }
                }
            }
            None => {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(ApiError::MissingParameters.to_string())
                    .unwrap();
            }
        }
        for (id, question) in questions.iter() {
            let id_index = id.0.parse::<usize>().unwrap();
            if id_index >= start_index && id_index <= end_index {
                result.insert(id.clone(), question.clone());
            }
        }
        Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string_pretty(&result).unwrap())
            .unwrap()
    }
}

/// API function to handle request to delete a question from the questions "Database"
#[instrument]
pub async fn delete_question(
    State(state): State<AppState>,
    Query(IdParam { id }): Query<IdParam>,
) -> impl IntoResponse {
    match id {
        Some(id) => {
            let question_id = QuestionId(id);
            match state.get_question(&question_id).await {
                Some(_) => {
                    state.delete_question(&question_id).await;
                    Response::builder()
                        .status(StatusCode::OK)
                        .body("Question deleted".to_string())
                        .unwrap()
                }
                None => Response::builder()
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

/// API function to handle request to update a question in the questions "Database"
#[instrument]
pub async fn put_question(
    State(state): State<AppState>,
    Json(question): Json<Question>,
) -> impl IntoResponse {
    let question_id = question.id.clone();
    match state.get_question(&question_id).await {
        Some(_) => {
            state.update_question(&question_id, question).await;
            Response::builder()
                .status(StatusCode::OK)
                .body("Question updated".to_string())
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(ApiError::QuestionNotFound.to_string())
            .unwrap(),
    }
}

/// Implementing the Display trait for the ApiError enum
/// #Example:
/// ```
/// let error = ApiError::MissingParameters;
/// println!("{}", error);
/// ```
///
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApiError::ParseError(e) => write!(f, "Cannot parse parameter: {}", e),
            ApiError::MissingParameters => write!(f, "Missing parameter"),
            ApiError::QuestionNotFound => write!(f, "Question not found"),
        }
    }
}

/// A parameter struct for the question id
///
/// This struct is used to get the id of a question from the query parameters
/// ##Example:
/// ```
/// {
///  "id": "1"
/// }
#[derive(Debug, Serialize, Deserialize)]
pub struct IdParam {
    pub id: Option<String>,
}

/// Function to post a question to the "database"
///
/// Currently only modifies the state of the application by adding a question to the questions hashmap, but will add write to file soon
#[instrument]
pub async fn post_question(
    State(state): State<AppState>,
    Json(question): Json<Question>,
) -> impl IntoResponse {
    state.add_question(question).await;
    Response::builder()
        .status(StatusCode::OK)
        .body("Question added".to_string())
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to add question".into())
                .unwrap()
        })
}

/// An enum to represent the possible errors that can occur in the API
///
/// #Example:
/// ```
/// ApiError::ParseError(std::num::ParseIntError) // When a parameter cannot be parsed
/// ApiError::MissingParameters // When a required parameter is missing
/// ApiError::QuestionNotFound // When a question is not found
/// ```
#[derive(Debug)]
pub enum ApiError {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    QuestionNotFound,
}

/// Implementing the IntoResponse trait for the ApiError enum
///
/// #Example:
///
/// ```
/// let error = ApiError::MissingParameters;
/// let response = error.into_response();
/// ```
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::ParseError(_) => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Cannot parse parameter".into())
                .unwrap(),
            ApiError::MissingParameters => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Missing parameter".into())
                .unwrap(),
            ApiError::QuestionNotFound => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Question not found".into())
                .unwrap(),
        }
    }
}
