use crate::api::{ApiContext, Json, Path, Permission};
use crate::error::ApiError;
use crate::middlewares::require_permission;
use axum::{handler::Handler, middleware::from_fn, routing::delete, Extension, Router};
use serde::Serialize;
use sqlx::query;

pub fn router() -> Router {
    let delete_message = delete_message
        .layer(from_fn(require_permission))
        .layer(Extension(Permission {
            read: false,
            write: false,
            delete: true,
        }));
    Router::new().route("/:space_id/messages/:msg_id", delete(delete_message))
}

#[derive(Serialize)]
struct DeleteMessageBody;

async fn delete_message(
    ctx: Extension<ApiContext>,
    Path((space_id, msg_id)): Path<(i32, i32)>,
) -> Result<Json<DeleteMessageBody>, ApiError> {
    query!(
        "DELETE FROM messages WHERE space_id = $1 AND msg_id = $2",
        space_id,
        msg_id,
    )
    .execute(&ctx.db)
    .await?;
    Ok(Json(DeleteMessageBody {}))
}
