use anyhow::anyhow;
use axum::{
    async_trait,
    body::Body,
    extract::{rejection::JsonRejection, FromRequest, RequestParts},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use sqlx::PgPool;

pub struct Json<T>(pub T);

#[async_trait]
impl<T> FromRequest<Body> for Json<T>
where
    T: DeserializeOwned,
{
    type Rejection = ApiError;
    async fn from_request(req: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let e = match rejection {
                    JsonRejection::JsonDataError(_) | JsonRejection::MissingJsonContentType(_) => {
                        ApiError::BadRequest
                    }
                    err => ApiError::ServerError(anyhow!(
                        "unknown error when parsing JSON payload: {}",
                        err
                    )),
                };
                Err(e)
            }
        }
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

pub type ApiResult<T> = Result<(StatusCode, T), ApiError>;

#[derive(Clone)]
pub struct ApiContext {
    pub db: PgPool,
}

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
