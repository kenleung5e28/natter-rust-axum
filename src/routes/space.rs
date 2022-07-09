use crate::api::{ApiContext, ApiError, Json, Query, IdPath};
use axum::{
    extract::{OriginalUri},
    http::StatusCode,
    routing::{get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_scalar};
use chrono::{DateTime, Duration, Utc};

pub fn router() -> Router {
    Router::new().route("/", post(create_space)).nest(
        "/:space_id/messages",
        Router::new()
            .route("/", post(post_message).get(find_messages))
            .route("/:msg_id", get(read_message)),
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
    uri: OriginalUri,
    Json(payload): Json<CreateSpacePayload>,
) -> Result<(StatusCode, Json<CreateSpaceBody>), ApiError> {
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
            uri: format!("{}/{}", uri.0, space_id),
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
    IdPath(space_id): IdPath<i32>,
    OriginalUri(uri): OriginalUri,
    Json(payload): Json<PostMessagePayload>,
) -> Result<(StatusCode, Json<PostMessageBody>), ApiError> {
    let author = payload.author;
    let message = payload.message;
    let msg_id = query_scalar!(
        "INSERT INTO messages (space_id, author, msg_text) VALUES ($1, $2, $3) RETURNING msg_id",
        space_id,
        author,
        message
    )
    .fetch_one(&ctx.db)
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(PostMessageBody {
            uri: format!("{}/{}", uri, msg_id),
        })
    ))
}

#[derive(Serialize)]
struct ReadMessageBody {
    author: String,
    message: String,
    time: DateTime<Utc>,
    uri: String,
}

async fn read_message(
    ctx: Extension<ApiContext>,
    IdPath((space_id, msg_id)): IdPath<(i32, i32)>,
    OriginalUri(uri): OriginalUri,
) -> Result<Json<ReadMessageBody>, ApiError> {
    let result = query!(
        "SELECT space_id, msg_id, author, msg_time, msg_text FROM messages WHERE space_id = $1 AND msg_id = $2",
        space_id, 
        msg_id,
    )
    .fetch_optional(&ctx.db)
    .await?;
    match result {
        Some(record) => Ok(Json(ReadMessageBody {
            author: record.author,
            message: record.msg_text,
            time: record.msg_time,
            uri: uri.to_string(),
        })),
        None => Err(ApiError::NotFound),
    }
}

#[derive(Deserialize)]
struct FindMessagesParam {
    since: Option<DateTime<Utc>>,
}

async fn find_messages(
    ctx: Extension<ApiContext>,
    IdPath(space_id): IdPath<i32>,
    Query(param): Query<FindMessagesParam>,
) -> Result<Json<Vec<i32>>, ApiError> {
    let msg_time = param.since
        .unwrap_or(Utc::now() - Duration::days(1));
    let result = query_scalar!(
        "SELECT msg_id FROM messages WHERE space_id = $1 and msg_time >= $2",
        space_id,
        msg_time,
    )
    .fetch_all(&ctx.db)
    .await?;
    Ok(Json(result))
}
