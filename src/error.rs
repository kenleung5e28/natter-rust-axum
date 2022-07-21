use axum::response::{IntoResponse, Json, Response};
use http::{
    header::{HeaderValue, RETRY_AFTER, WWW_AUTHENTICATE},
    StatusCode,
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("resource not found")]
    NotFound,
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("only support application/json content type")]
    OnlySupportJsonContentType,
    #[error("too many requests")]
    TooManyRequests,
    #[error("authentication required")]
    AuthenticationRequired,
    #[error("internal server error")]
    ServerError(#[from] anyhow::Error),
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::OnlySupportJsonContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ApiError::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ApiError::AuthenticationRequired => StatusCode::UNAUTHORIZED,
            ApiError::DatabaseError(e) => {
                dbg!(e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ApiError::ServerError(e) => {
                dbg!(e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        let mut response = (
            status_code,
            Json(json!({
                "message": &self.to_string(),
            })),
        )
            .into_response();
        if let ApiError::TooManyRequests = &self {
            response
                .headers_mut()
                .insert(RETRY_AFTER, HeaderValue::from_static("2"));
        }
        if let ApiError::AuthenticationRequired = &self {
            response.headers_mut().insert(
                WWW_AUTHENTICATE,
                HeaderValue::from_static("Basic realm=\"/\", charset=\"UTF-8\""),
            );
        }
        response
    }
}
