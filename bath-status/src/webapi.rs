use crate::database::*;
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};

pub async fn daemon(){
    let app = Router::new()
        .route("/", get(root));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}