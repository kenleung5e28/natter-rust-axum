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

#[derive(Deserialize)]
struct CreateSpacePayload {
    name: String,
    owner: String,
}

#[derive(Serialize)]
struct CreateSpaceBody {
    name: String,
    uri: String,
}

async fn create_space(
    ctx: Extension<ApiContext>,
    Json(req): Json<CreateSpacePayload>,
) -> Result<Json<CreateSpaceBody>, ApiError> {
    let CreateSpacePayload { name, owner } = req;

    Ok(Json(CreateSpaceBody {
        name,
        uri: format!("/spaces/{}", 1997),
    }))
}
