use crate::routes::ApiContext;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};

pub fn router() -> Router {
    Router::new().route("/", post(create_space))
}

async fn create_space(ctx: Extension<ApiContext>) {}
