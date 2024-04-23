use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderValue;
use axum::routing::{delete, put};
use axum::{
    extract::{Json, Query, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::usize;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

/// A question struct
///
/// This struct represents a question that can be asked and (future) answered via the API
/// ##Example:
/// ```
/// {
///    "id": "1",
///    "title": "What is cargo toml?",
///    "content": "I want to know what toml is and how it relates to cargo. Can someone explain?",
///    "tags": ["rust", "toml", "cargo"]
/// }
/// ```
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

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
struct Pagination {
    start: Option<QuestionId>,
    end: Option<QuestionId>,
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
struct IdParam {
    id: Option<String>,
}

/// A question id struct
///
/// This struct is used to represent the id of a question. Why, because the book said so, that's why.
/// ##Example:
/// ```
/// {
/// "id": "1"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
struct QuestionId(String);

/// Function to post a question to the "database"
///
/// Currently only modifies the state of the application by adding a question to the questions hashmap, but will add write to file soon
async fn post_question(
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
enum ApiError {
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

/// Implementing the Display trait for the ApiError enum
/// #Example:
/// ```
/// let error = ApiError::MissingParameters;
/// println!("{}", error);
/// ```
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApiError::ParseError(e) => write!(f, "Cannot parse parameter: {}", e),
            ApiError::MissingParameters => write!(f, "Missing parameter"),
            ApiError::QuestionNotFound => write!(f, "Question not found"),
        }
    }
}

/// Application state struct
/// This struct is used to hold the state of the application, which is currently only the questions for the API
#[derive(Clone)]
struct AppState {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
}

/// Implementing the AppState struct with basic functions to use for API and state management operations
impl AppState {
    fn new() -> Self {
        AppState {
            questions: Arc::new(RwLock::new(self::AppState::init())),
        }
    }

    /// Function to initialize the questions hashmap by reading in the questions from a json file
    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        let questions: HashMap<QuestionId, Question> =
            serde_json::from_str::<HashMap<QuestionId, Question>>(file)
                .unwrap()
                .into_iter()
                .collect();
        questions
    }

    /// Function to get a question from the questions hashmap
    async fn get_question(&self, id: &QuestionId) -> Option<Question> {
        self.questions.read().await.get(id).cloned()
    }

    /// Function to add a question to the questions hashmap
    async fn add_question(self, question: Question) -> Self {
        self.questions
            .write()
            .await
            .insert(question.id.clone(), question);
        self
    }

    /// Function to delete a question from the questions hashmap
    async fn delete_question(self, id: &QuestionId) -> Self {
        self.questions.write().await.remove(id);
        self
    }

    /// Function to update a question in the questions hashmap
    async fn update_question(self, id: &QuestionId, question: Question) -> Self {
        self.questions.write().await.insert(id.clone(), question);
        self
    }
}

impl Clone for Question {
    fn clone(&self) -> Self {
        Question {
            id: self.id.clone(),
            title: self.title.clone(),
            content: self.content.clone(),
            tags: self.tags.clone(),
        }
    }
}

/// API function to get all questions or a range of questions from the questions hashmap
/// 
async fn get_questions(
    State(state): State<AppState>,
    Query(Pagination { start, end }): Query<Pagination>,
) -> impl IntoResponse {
    if start.is_none() && end.is_none() {
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


/// API function to get a single question from the questions 
async fn get_question(
    State(state): State<AppState>,
    Query(IdParam { id }): Query<IdParam>,
) -> impl IntoResponse {
    match id {
        Some(id) => {
            let question_id = QuestionId(id);
            match state.get_question(&question_id).await {
                Some(question) => Response::builder()
                    .status(StatusCode::OK)
                    .body(serde_json::to_string_pretty(&question).unwrap())
                    .unwrap(),
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

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(QuestionId(id.to_string())),
            true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}

/// API function to handle a not found error instead of other hard coding stuff. Will expand further in the future
async fn handle_not_found() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("Not Found".to_string())
        .unwrap()
}

/// API function to handle request to update a question in the questions "Database"
async fn put_question(
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

/// API function to handle request to delete a question from the questions "Database"
async fn delete_question(
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

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE])
        .allow_credentials(true)
        .max_age(Duration::from_secs(60) * 10); // 10 minutes, was just toying with cors
    let state = AppState::new();
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/questions", get(get_questions))
        .route("/questions", post(post_question))
        .route("/question", get(get_question))
        .route("/questions/:id", put(put_question))
        .route("/questions/:id", delete(delete_question))
        .route("/answers", post(handle_not_found))
        .layer(cors)
        .with_state(state)
        .fallback(handle_not_found);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
