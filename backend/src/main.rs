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
use sqlx::{self, postgres::PgPool, Pool, Row};
use std::error::Error;
use std::str::FromStr;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::trace;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, ToSchema};
extern crate thiserror;
mod api;
mod auth;
mod bad_words_api;
mod database;
mod question;
mod web;
use crate::api::{
    delete_account, delete_answer, delete_question, get_account, get_answers, get_questions,
    post_account, post_answer, post_question, put_account, put_answer, put_question,
};
use crate::auth::login;
use crate::question::{Question, QuestionId};
use crate::web::{get_entry_point, get_question};
use database::AppState;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

/// API function to handle a not found error instead of other hard coding stuff. Will expand further in the future
async fn handle_not_found() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("Not Found".to_string())
        .unwrap()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "questions=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    // Credit to course knock-knock for the trace layer
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE])
        .allow_credentials(true)
        .max_age(Duration::from_secs(60) * 10); // 10 minutes, was just toying with cors
    let swagger_ui =
        SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api::ApiDoc::openapi());
    let redoc_ui = Redoc::with_url("/redoc", api::ApiDoc::openapi());
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnSessionEnd);
    let state = AppState::new().await.unwrap();
    let app = Router::new()
        .route("/", get(get_entry_point))
        .route("/questions", get(get_questions))
        .route("/questions", post(post_question))
        .route("/question", get(get_question))
        .route("/questions", put(put_question))
        .route("/questions", delete(delete_question))
        // The following routes are for the answers portion of the API
        .route("/answers", post(post_answer))
        .route("/answers", delete(delete_answer))
        .route("/answers", put(put_answer))
        .route("/answers", get(get_answers))
        // The following routes are for the accounts portion of the API
        .route("/accounts", post(post_account))
        .route("/accounts", delete(delete_account))
        .route("/accounts", put(put_account))
        .route("/accounts", get(get_account))
        // auth stuffs
        .route("/login", get(login))
        // Layers
        .merge(swagger_ui)
        .merge(redoc_ui)
        .merge(rapidoc_ui)
        .layer(cors)
        .layer(trace_layer)
        .layer(session_layer)
        .with_state(state)
        .fallback(handle_not_found);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    tracing::debug!("serving {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
