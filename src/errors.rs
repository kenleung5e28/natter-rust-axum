use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("bad request")]
    BadRequest,
    #[error("server error has occurred")]
    ServerError(#[from] anyhow::Error),
    #[error("database error has occurred")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest => StatusCode::BAD_REQUEST,
            ApiError::ServerError(_) | ApiError::DatabaseError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        (
            status_code,
            Json(json!({
                "message": &self.to_string(),
            })),
        )
            .into_response()
    }
}
