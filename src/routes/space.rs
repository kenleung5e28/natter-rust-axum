use crate::routes::{ApiContext, ApiResult};
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::query;

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
) -> ApiResult<Json<CreateSpaceBody>> {
    let name = req.name;
    let owner = req.owner;
    let result = query!(
        "INSERT INTO spaces (name, owner) VALUES ($1, $2) RETURNING space_id",
        name,
        owner
    )
    .fetch_one(&ctx.db)
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreateSpaceBody {
            name,
            uri: format!("/spaces/{}", result.space_id),
        }),
    ))
}
