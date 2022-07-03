use crate::errors::ApiError;
use crate::routes::ApiContext;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};

pub fn router() -> Router {
    Router::new().route("/", post(create_space))
}

async fn create_space(ctx: Extension<ApiContext>) {}
