use tracing::{info, instrument};

use crate::database::*;
use crate::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_questions,
        delete_question,
        put_question,
        post_question
    ),
    components(
        schemas(Question, ApiError),
    ),
    tags(
        (name = "Question", description = "Questions API")
    )
)]
pub struct ApiDoc;

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
#[utoipa::path(get, path = "/questions", responses((
    status = 200,
    description = "Returns all questions or a range of questions",
    body = [Question]
),
(status = 204, description = "Questions db is empty", body = ApiError)))]
#[instrument]
pub async fn get_questions(
    State(state): State<AppState>,
    Query(Pagination { start, end }): Query<Pagination>,
) -> impl IntoResponse {
    let questions = state.get_all_questions().await.unwrap();
    if start.is_none() && end.is_none() {
        info!("Getting all questions");
        Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string_pretty(&questions.clone()).unwrap())
            .unwrap()
    } else {
        let mut result = Vec::new();
        let start_index = match start {
            Some(s) => s.0,
            None => {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(ApiError::MissingParameters.to_string())
                    .unwrap();
            }
        };
        let end_index = match end {
            Some(s) => s.0,
            None => {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(ApiError::MissingParameters.to_string())
                    .unwrap();
            }
        };
        for question in questions {
            if question.id.0 >= start_index && question.id.0 <= end_index {
                result.push(question);
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
#[utoipa::path(delete, path = "/questions/:id", responses((
    status = 200,
    description = "Question deleted"
),
(status = 404, description = "Question not found", body = ApiError)))]
pub async fn delete_question(
    State(state): State<AppState>,
    Query(IdParam { id }): Query<IdParam>,
) -> impl IntoResponse {
    let question_id = QuestionId(id.unwrap());
    if state.get_question(&question_id).await.is_err() {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(ApiError::QuestionNotFound.to_string())
            .unwrap();
    }
    match state.delete_question(&question_id).await {
        Ok(_) => (),
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to delete question".into())
                .unwrap();
        }
    }
    Response::builder()
        .status(StatusCode::OK)
        .body("Question deleted".to_string())
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to delete question".into())
                .unwrap()
        })
}

/// API function to handle request to update a question in the questions "Database"
#[instrument]
#[utoipa::path(put, path = "/questions/:id", responses((
    status = 200,
    description = "Question updated"
),
(status = 404, description = "Question not found", body = ApiError)))]
pub async fn put_question(
    State(state): State<AppState>,
    Query(IdParam { id }): Query<IdParam>,
    Json(question): Json<question::UpdateQuestion>,
) -> impl IntoResponse {
    let question_id = match id {
        Some(id) => QuestionId(id),
        None => match question.id {
            Some(id) => id,
            None => {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(ApiError::MissingParameters.to_string())
                    .unwrap();
            }
        },
    };
    if state.get_question(&question_id).await.is_err() {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(ApiError::QuestionNotFound.to_string())
            .unwrap();
    }
    let updated_question = Question {
        id: question_id.clone(),
        title: question.title,
        content: question.content,
        tags: question.tags,
    };
    match state.update_question(&question_id, updated_question).await {
        Ok(_) => (),
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to update question".into())
                .unwrap();
        }
    }
    Response::builder()
        .status(StatusCode::OK)
        .body("Question updated".to_string())
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to update question".into())
                .unwrap()
        })
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
    pub id: Option<i32>,
}

/// Function to post a question to the "database"
///
/// Currently only modifies the state of the application by adding a question to the questions hashmap, but will add write to file soon
#[instrument]
#[utoipa::path(post, path = "/questions", responses((
    status = 200,
    description = "Question added"
),
(status = 500, description = "Failed to add question", body = ApiError)))]
pub async fn post_question(
    State(state): State<AppState>,
    Json(question): Json<Question>,
) -> impl IntoResponse {
    match state.add_question(question).await {
        Ok(_) => {
            return Response::builder()
                .status(StatusCode::OK)
                .body("Question added".to_string())
                .unwrap()
        }
        Err(error) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string())
                .unwrap()
        }
    }
}

/// An enum to represent the possible errors that can occur in the API
///
/// #Example:
/// ```
/// ApiError::ParseError(std::num::ParseIntError) // When a parameter cannot be parsed
/// ApiError::MissingParameters // When a required parameter is missing
/// ApiError::QuestionNotFound // When a question is not found
/// ```
#[derive(Debug, ToSchema, thiserror::Error)]
pub enum ApiError {
    #[error("Missing parameter")]
    MissingParameters,
    #[error("Question not found")]
    QuestionNotFound,
    #[error("Database error: {0}")]
    DatabaseError(String),
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
            ApiError::MissingParameters => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Missing parameter".to_string().into())
                .unwrap(),
            ApiError::QuestionNotFound => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Question not found".to_string().into())
                .unwrap(),
            ApiError::DatabaseError(error) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string().into())
                .unwrap(),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError::DatabaseError(e.to_string())
    }
}
