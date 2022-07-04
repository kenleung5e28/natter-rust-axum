use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("request path not found")]
    NotFound,
    #[error("bad request")]
    BadRequest,
    #[error("an internal server error occurred")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::BadRequest => StatusCode::BAD_REQUEST,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
