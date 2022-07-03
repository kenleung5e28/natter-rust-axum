use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

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
    fn into_response(self) -> Response {}
}
