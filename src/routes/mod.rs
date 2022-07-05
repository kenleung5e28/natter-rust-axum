use crate::errors::ApiError;
use axum::http::StatusCode;
use sqlx::PgPool;

pub mod space;

pub type ApiResult<T> = Result<(StatusCode, T), ApiError>;

#[derive(Clone)]
pub struct ApiContext {
    pub db: PgPool,
}
