use anyhow::anyhow;
use axum::{
    async_trait,
    body::Body,
    extract::{
        rejection::{JsonRejection, PathRejection, QueryRejection},
        FromRequest, RequestParts,
    },
    http::{
        header::HeaderValue,
        header::{LOCATION, RETRY_AFTER},
        StatusCode,
    },
    response::{IntoResponse, Response},
};
use governor::{clock::DefaultClock, state::direct::NotKeyed, state::InMemoryState, RateLimiter};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

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
                    JsonRejection::MissingJsonContentType(_) => ApiError::BadRequest(
                        "request missing the application/json content-type".to_string(),
                    ),
                    JsonRejection::JsonSyntaxError(err) => ApiError::BadRequest(format!(
                        "JSON payload has syntax error: {}",
                        err.to_string()
                    )),
                    JsonRejection::JsonDataError(err) => ApiError::BadRequest(format!(
                        "invalid request JSON payload: {}",
                        err.to_string()
                    )),
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

pub struct CreatedJson<T>(pub String, pub T);

impl<T> IntoResponse for CreatedJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match HeaderValue::from_str(&self.0) {
            Ok(value) => {
                let mut response = Json(self.1).into_response();
                response.headers_mut().insert(LOCATION, value);
                (StatusCode::CREATED, response).into_response()
            }
            Err(e) => ApiError::ServerError(anyhow!(e)).into_response(),
        }
    }
}

pub struct Query<T>(pub T);

#[async_trait]
impl<T> FromRequest<Body> for Query<T>
where
    T: DeserializeOwned,
{
    type Rejection = ApiError;
    async fn from_request(req: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {
        match axum::extract::Query::<T>::from_request(req).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let e = match rejection {
                    QueryRejection::FailedToDeserializeQueryString(_) => {
                        ApiError::BadRequest("Invalid query parameter: {}".to_string())
                    }
                    err => ApiError::ServerError(anyhow!(
                        "unknown error when parsing query parameter: {}",
                        err
                    )),
                };
                Err(e)
            }
        }
    }
}

pub struct IdPath<T>(pub T);

#[async_trait]
impl<T> FromRequest<Body> for IdPath<T>
where
    T: Send + DeserializeOwned,
{
    type Rejection = ApiError;
    async fn from_request(req: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request(req).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let e = match rejection {
                    PathRejection::FailedToDeserializePathParams(_) => ApiError::NotFound,
                    err => {
                        ApiError::ServerError(anyhow!("unknown error when parsing path: {}", err))
                    }
                };
                Err(e)
            }
        }
    }
}

#[derive(Clone)]
pub struct ApiContext {
    pub db: PgPool,
    pub limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("resource not found")]
    NotFound,
    #[error("{0}")]
    BadRequest(String),
    #[error("only support application/json content type")]
    OnlySupportJsonContentType,
    #[error("too many requests")]
    TooManyRequests,
    #[error("server error: {0}")]
    ServerError(#[from] anyhow::Error),
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::OnlySupportJsonContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ApiError::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ApiError::ServerError(_) | ApiError::DatabaseError(_) => {
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
        response
    }
}
