use crate::routes::{ApiContext, ApiResult};
use axum::{
    extract::{MatchedPath, Path},
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_scalar};

pub fn router() -> Router {
    Router::new().route("/", post(create_space)).nest(
        "/:space_id/messages",
        Router::new().route("/", post(post_message)),
    )
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
    path: MatchedPath,
    Json(payload): Json<CreateSpacePayload>,
) -> ApiResult<Json<CreateSpaceBody>> {
    let name = payload.name;
    let owner = payload.owner;
    let space_id = query_scalar!(
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
            uri: format!("{}/{}", path.as_str(), space_id),
        }),
    ))
}

#[derive(Deserialize)]
struct PostMessagePayload {
    author: String,
    message: String,
}

#[derive(Serialize)]
struct PostMessageBody {
    uri: String,
}

async fn post_message(
    ctx: Extension<ApiContext>,
    Path(space_id): Path<i32>,
    path: MatchedPath,
    Json(payload): Json<PostMessagePayload>,
) -> ApiResult<Json<PostMessageBody>> {
    let author = payload.author;
    let message = payload.message;
    let msg_id = query_scalar!(
        r#"
        INSERT INTO messages (space_id, author, msg_text) VALUES ($1, $2, $3) RETURNING msg_id
    "#,
        space_id,
        author,
        message
    )
    .fetch_one(&ctx.db)
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(PostMessageBody {
            uri: format!("{}/{}", path.as_str(), msg_id),
        }),
    ))
}
