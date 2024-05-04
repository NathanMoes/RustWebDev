use tracing::{info, instrument};

use crate::database::*;
use crate::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_questions,
        delete_question,
        put_question,
        post_question,
        post_account,
        get_account,
        delete_account,
        put_account,
        get_answers,
        delete_answer,
        put_answer,
        post_answer,
    ),
    components(
        schemas(Question, ApiError, Account, Answer),
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
    body = None
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
    description = "Question updated",
    body = UpdateQuestion
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

/// A parameter struct for the user email
///
/// This struct is used to get the user email from the query parameters
/// ##Example:
/// ```
/// {
///  "email": "moes@pdx.edu"
/// }
#[derive(Debug, Serialize, Deserialize)]
pub struct UserAccountInfo {
    pub email: Option<String>,
    pub password: Option<String>,
}

/// Function to post a question to the "database"
///
/// Currently only modifies the state of the application by adding a question to the questions hashmap, but will add write to file soon
#[instrument]
#[utoipa::path(post, path = "/questions", responses((
    status = 200,
    description = "Question added",
    body = Question
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
            tracing::event!(tracing::Level::ERROR, "{:?}", error);
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string())
                .unwrap();
        }
    }
}

/// Function to create an account in the "database"
///
#[instrument]
#[utoipa::path(post, path = "/account", responses((
    status = 200,
    description = "Account added",
    body = None
),
(status = 500, description = "Failed to add account", body = ApiError)))]
pub async fn post_account(
    State(state): State<AppState>,
    Json(account): Json<Account>,
) -> impl IntoResponse {
    match state.add_account(account).await {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .body("Account added".to_string())
            .unwrap(),
        Err(error) => {
            tracing::event!(tracing::Level::ERROR, "{:?}", error);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string())
                .unwrap()
        }
    }
}

/// Function to get an account from the "database"
#[instrument]
#[utoipa::path(get, path = "/account", responses((
    status = 200,
    description = "Returns all accounts",
    body = None
),
(status = 404, description = "Account not found", body = ApiError)))]
pub async fn get_account(
    State(state): State<AppState>,
    Query(UserAccountInfo { email, password }): Query<UserAccountInfo>,
) -> impl IntoResponse {
    let email = match email {
        Some(email) => email,
        None => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(ApiError::MissingParameters.to_string())
                .unwrap();
        }
    };
    match state.get_account(&email).await {
        Ok(account) => Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string_pretty(&account).unwrap())
            .unwrap(),
        Err(error) => {
            tracing::event!(tracing::Level::ERROR, "{:?}", error);
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(ApiError::AccountNotFound.to_string())
                .unwrap();
        }
    }
}

/// Function to delete an account from the "database"
#[instrument]
#[utoipa::path(delete, path = "/account", responses((
    status = 200,
    description = "Account deleted",
    body = None
),
(status = 404, description = "Account not found", body = ApiError)))]
pub async fn delete_account(
    State(state): State<AppState>,
    Query(UserAccountInfo { email, password }): Query<UserAccountInfo>,
) -> impl IntoResponse {
    let email = match email {
        Some(email) => email,
        None => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(ApiError::MissingParameters.to_string())
                .unwrap();
        }
    };
    match state.delete_account(&email).await {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .body("Account deleted".to_string())
            .unwrap(),
        Err(error) => {
            tracing::event!(tracing::Level::ERROR, "{:?}", error);
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(ApiError::AccountNotFound.to_string())
                .unwrap();
        }
    }
}

/// Function to update an account in the "database"
#[instrument]
#[utoipa::path(put, path = "/account", responses((
    status = 200,
    description = "Account updated",
    body = None
),
(status = 404, description = "Account not found", body = ApiError)))]
pub async fn put_account(
    State(state): State<AppState>,
    Query(UserAccountInfo { email, password }): Query<UserAccountInfo>,
    Json(account): Json<Account>,
) -> impl IntoResponse {
    let email = match email {
        Some(email) => email,
        None => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(ApiError::MissingParameters.to_string())
                .unwrap();
        }
    };
    match state.update_account(&email, account).await {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .body("Account updated".to_string())
            .unwrap(),
        Err(error) => {
            tracing::event!(tracing::Level::ERROR, "{:?}", error);
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(ApiError::AccountNotFound.to_string())
                .unwrap();
        }
    }
}

/// Function to get an answer from the "database"
#[instrument]
#[utoipa::path(get, path = "/answers", responses((
    status = 200,
    description = "Returns all answers for a question",
    body = None
),
(status = 404, description = "Question not found", body = ApiError)))]
pub async fn get_answers(
    State(state): State<AppState>,
    Query(IdParam { id }): Query<IdParam>,
) -> impl IntoResponse {
    let question_id = QuestionId(id.unwrap());
    match state.get_answers(&question_id).await {
        Ok(answer) => Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string_pretty(&answer).unwrap())
            .unwrap(),
        Err(error) => {
            tracing::event!(tracing::Level::ERROR, "{:?}", error);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(ApiError::AnswerNotFound.to_string())
                .unwrap()
        }
    }
}

/// Function to delete an answer from the "database"
#[instrument]
#[utoipa::path(delete, path = "/answers/:id", responses((
    status = 200,
    description = "Answer deleted",
    body = None
),
(status = 500, description = "Failed to delete answer", body = ApiError)))]
pub async fn delete_answer(
    State(state): State<AppState>,
    Query(IdParam { id }): Query<IdParam>,
) -> impl IntoResponse {
    let answer_id = QuestionId(id.unwrap());
    match state.delete_answer(&answer_id).await {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .body("Answer deleted".to_string())
            .unwrap(),
        Err(error) => {
            tracing::event!(tracing::Level::ERROR, "{:?}", error);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to delete answer".to_string())
                .unwrap()
        }
    }
}

/// Function to update an answer in the "database"
#[instrument]
#[utoipa::path(put, path = "/answers/:id", responses((
    status = 200,
    description = "Answer updated",
    body = None
),
(status = 500, description = "Failed to update answer", body = ApiError)))]
pub async fn put_answer(
    State(state): State<AppState>,
    Query(IdParam { id }): Query<IdParam>,
    Json(answer): Json<Answer>,
) -> impl IntoResponse {
    let answer_id = QuestionId(id.unwrap());
    match state.update_answer(&answer_id, answer).await {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .body("Answer updated".to_string())
            .unwrap(),
        Err(error) => {
            tracing::event!(tracing::Level::ERROR, "{:?}", error);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string())
                .unwrap()
        }
    }
}

/// Function to create an answer in the "database"
#[instrument]
#[utoipa::path(post, path = "/answers", responses((
    status = 200,
    description = "Answer added",
    body = None
),
(status = 500, description = "Failed to add answer", body = ApiError)))]
pub async fn post_answer(
    State(state): State<AppState>,
    Json(answer): Json<Answer>,
) -> impl IntoResponse {
    match state.add_answer(answer).await {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .body("Answer added".to_string())
            .unwrap(),
        Err(error) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(error.to_string())
            .unwrap(),
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
    #[error("Account not found")]
    AccountNotFound,
    #[error("Answer not found")]
    AnswerNotFound,
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
            ApiError::AccountNotFound => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Account not found".to_string().into())
                .unwrap(),
            ApiError::AnswerNotFound => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Answer not found".to_string().into())
                .unwrap(),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError::DatabaseError(e.to_string())
    }
}
