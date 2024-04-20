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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Pagination {
    start: Option<QuestionId>,
    end: Option<QuestionId>,
}

#[derive(Debug, Serialize, Deserialize)]
struct IdParam {
    id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
struct QuestionId(String);

impl Question {
    fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}

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

#[derive(Debug)]
enum ApiError {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    QuestionNotFound,
}

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

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApiError::ParseError(e) => write!(f, "Cannot parse parameter: {}", e),
            ApiError::MissingParameters => write!(f, "Missing parameter"),
            ApiError::QuestionNotFound => write!(f, "Question not found"),
        }
    }
}

#[derive(Clone)]
struct AppState {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            questions: Arc::new(RwLock::new(self::AppState::init())),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        let questions: HashMap<QuestionId, Question> =
            serde_json::from_str::<HashMap<QuestionId, Question>>(file)
                .unwrap()
                .into_iter()
                .collect();
        questions
    }

    async fn get_question(&self, id: &QuestionId) -> Option<Question> {
        self.questions.read().await.get(id).cloned()
    }

    async fn add_question(self, question: Question) -> Self {
        self.questions
            .write()
            .await
            .insert(question.id.clone(), question);
        self
    }

    async fn delete_question(self, id: &QuestionId) -> Self {
        self.questions.write().await.remove(id);
        self
    }

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

async fn update_question(
    State(state): State<AppState>,
    Json(question): Json<Question>,
    id: QuestionId,
) -> impl IntoResponse {
    match state.get_question(&id).await {
        Some(_) => {
            state.update_question(&id, question).await;
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

async fn handle_not_found() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("Not Found".to_string())
        .unwrap()
}

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

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE])
        .allow_credentials(true)
        .max_age(Duration::from_secs(60) * 10);
    let state = AppState::new();
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/questions", get(get_questions))
        .route("/questions", post(post_question))
        .route("/question", get(get_question))
        .route("/questions/:id", put(put_question))
        .route("/questions/:id", delete(handle_not_found))
        .route("/answers", post(handle_not_found))
        .layer(cors)
        .with_state(state)
        .fallback(handle_not_found);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
