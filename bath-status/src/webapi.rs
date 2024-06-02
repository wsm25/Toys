use std::{rc::Rc, sync::Arc};

use crate::database::*;
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde_json::{Value, json};

pub async fn daemon(){
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    let app = Router::new()
        .route("/", get(root))
        .with_state(Arc::new(()
        ));
        
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Json<Value> {
    Json(json!({ "hello": "world!" }))
}