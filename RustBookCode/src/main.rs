use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderValue;
use axum::routing::{delete, put};
use axum::Json;
use axum::{
    extract::State,
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
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tower_http::cors::CorsLayer;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
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
    state.questions.lock().unwrap().insert(question.id.clone(), question);
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

#[derive(Clone)]
struct AppState {
    questions: Arc<Mutex<HashMap<QuestionId, Question>>>,
}

impl AppState {
    fn new(questions: HashMap<QuestionId, Question>) -> Self {
        AppState {
            questions: Arc::new(Mutex::new(questions)),
        }
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

async fn get_questions(State(state): State<AppState>) -> String {
    let questions = state.questions.lock().unwrap();
    serde_json::to_string_pretty(&questions.clone()).unwrap()
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

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE])
        .allow_credentials(true)
        .max_age(Duration::from_secs(60) * 10);
    // Credit for questions from github co-pilot
    let questions = vec![
        Question::new(
            QuestionId("1".to_string()),
            "What is Rust?".to_string(),
            "Rust is a systems programming language".to_string(),
            Some(vec!["rust".to_string(), "programming".to_string()]),
        ),
        Question::new(
            QuestionId("2".to_string()),
            "What is Tokio?".to_string(),
            "Tokio is an asynchronous runtime for Rust".to_string(),
            Some(vec!["tokio".to_string(), "asynchronous".to_string()]),
        ),
        Question::new(
            QuestionId("3".to_string()),
            "What is Axum?".to_string(),
            "Axum is a web framework based on hyper and tower".to_string(),
            Some(vec!["axum".to_string(), "web".to_string()]),
        ),
    ];
    let state = AppState::new(questions.into_iter().map(|item| (item.id.clone(), item)).collect());
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/questions", get(get_questions).with_state(state.clone()))
        .route("/questions", post(post_question).with_state(state))
        .route("/questions/:id", put(handle_not_found))
        .route("/questions/:id", delete(handle_not_found))
        .route("/answers", post(handle_not_found))
        .layer(cors)
        .fallback(handle_not_found);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
