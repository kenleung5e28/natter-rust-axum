use crate::api::{ApiContext, CreatedJson, Json, Query, Path, AuthContext, Permission};
use crate::error::ApiError;
use axum::{
    extract::{OriginalUri},
    routing::{get, post},
    Extension, Router,
    middleware::from_fn,
    handler::Handler,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_scalar};
use chrono::{DateTime, Duration, Utc};
use validator::Validate;
use crate::routes::USER_REGEX;
use crate::middlewares::{require_permission, require_authentication};

pub fn router() -> Router {
    let create_space = create_space.layer(from_fn(require_authentication));
    let post_message = post_message.layer(from_fn(require_permission))
    .layer(Extension(Permission { read: false, write: true, delete: false, }));
    let find_messages = find_messages.layer(from_fn(require_permission))
    .layer(Extension(Permission { read: true, write: false, delete: false, }));
    let read_message = read_message.layer(from_fn(require_permission))
    .layer(Extension(Permission { read: true, write: false, delete: false, }));
    Router::new().route("/", post(create_space)).nest(
        "/:space_id/messages",
        Router::new()
            .route("/", post(post_message).get(find_messages))
            .route("/:msg_id", get(read_message)),
    )
}

#[derive(Deserialize, Validate)]
struct CreateSpacePayload {
    #[validate(length(max = 255))]
    name: String,
    #[validate(regex = "USER_REGEX")]
    owner: String,
}

#[derive(Serialize)]
struct CreateSpaceBody {
    name: String,
    uri: String,
}

async fn create_space(
    ctx: Extension<ApiContext>,
    auth_ctx: Extension<AuthContext>,
    OriginalUri(uri): OriginalUri,
    Json(payload): Json<CreateSpacePayload>,
) -> Result<CreatedJson<CreateSpaceBody>, ApiError> {
    if let Err(e) = payload.validate() {
        if e.errors().contains_key("owner") {
            return Err(ApiError::BadRequest("invalid user name".to_string()));
        }
        if e.errors().contains_key("name") {
            return Err(ApiError::BadRequest("name too long".to_string()));
        }
    }
    let name = payload.name;
    let owner = payload.owner;
    let is_owner_match = match &auth_ctx.subject {
        None => false,
        Some(subject) => *subject == owner,
    };
    if !is_owner_match {
        return Err(ApiError::BadRequest("owner must match authenticated user".to_string()));
    }
    let mut transaction = ctx.db.begin().await?;
    let space_id = query_scalar!(
        "INSERT INTO spaces (name, owner) VALUES ($1, $2) RETURNING space_id",
        name,
        owner
    )
    .fetch_one(&mut transaction)
    .await?;
    query!("INSERT INTO permissions (space_id, user_id, perms) VALUES ($1, $2, $3)", space_id, owner, "rwd").execute(&mut transaction).await?;
    transaction.commit().await?;
    let uri = format!("{}/{}", uri, space_id);
    Ok(
        CreatedJson(uri.clone(), CreateSpaceBody {
            name,
            uri,
        }),
    )
}

#[derive(Deserialize, Validate)]
struct PostMessagePayload {
    #[validate(regex = "USER_REGEX")]
    author: String,
    #[validate(length(max = 1024))]
    message: String,
}

#[derive(Serialize)]
struct PostMessageBody {
    uri: String,
}

async fn post_message(
    ctx: Extension<ApiContext>,
    auth_ctx: Extension<AuthContext>,
    Path(space_id): Path<i32>,
    OriginalUri(uri): OriginalUri,
    Json(payload): Json<PostMessagePayload>,
) -> Result<CreatedJson<PostMessageBody>, ApiError> {
    if let Err(e) = payload.validate() {
        if e.errors().contains_key("author") {
            return Err(ApiError::BadRequest("invalid user name".to_string()));
        }
        if e.errors().contains_key("message") {
            return Err(ApiError::BadRequest("message too long".to_string()));
        }
    }
    let author = payload.author;
    let message = payload.message;
    let is_author_match = match &auth_ctx.subject {
        None => false,
        Some(subject) => *subject == author,
    };
    if !is_author_match {
        return Err(ApiError::BadRequest("author must match authenticated user".to_string()));
    }
    let msg_id = query_scalar!(
        "INSERT INTO messages (space_id, author, msg_text) VALUES ($1, $2, $3) RETURNING msg_id",
        space_id,
        author,
        message
    )
    .fetch_one(&ctx.db)
    .await?;
    let uri = format!("{}/{}", uri, msg_id);
    Ok(
        CreatedJson(uri.clone(), PostMessageBody {
            uri,
        })
    )
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
    Path((space_id, msg_id)): Path<(i32, i32)>,
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
    Path(space_id): Path<i32>,
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
