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
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::{ trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod api;
mod database;
mod question;
mod web;
use crate::api::{delete_question, get_questions, post_question, put_question};
use crate::question::{Question, QuestionId};
use crate::web::{get_entry_point, get_question};
use database::AppState;

/// API function to handle a not found error instead of other hard coding stuff. Will expand further in the future
async fn handle_not_found() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("Not Found".to_string())
        .unwrap()
}

#[tokio::main]
async fn main() {
    // log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // log::info!("Starting server...");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "questions=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE])
        .allow_credentials(true)
        .max_age(Duration::from_secs(60) * 10); // 10 minutes, was just toying with cors
    let state = AppState::new();
    let app = Router::new()
        .route("/", get(get_entry_point))
        .route("/questions", get(get_questions))
        .route("/questions", post(post_question))
        .route("/question", get(get_question))
        .route("/questions/:id", put(put_question))
        .route("/questions/:id", delete(delete_question))
        .route("/answers", post(handle_not_found))
        .layer(cors)
        .layer(trace_layer)
        .with_state(state)
        .fallback(handle_not_found);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::debug!("serving {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
