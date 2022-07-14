use crate::api::{ApiContext, CreatedJson, Json};
use crate::error::ApiError;
use crate::routes::USER_REGEX;
use anyhow::{anyhow, Context};
use axum::{extract::OriginalUri, routing::post, Extension, Router};
use scrypt::password_hash::PasswordHasher;
use scrypt::{
    password_hash::{rand_core::OsRng, SaltString},
    Scrypt,
};
use serde::{Deserialize, Serialize};
use sqlx::query;
use validator::Validate;

pub fn router() -> Router {
    Router::new().route("/", post(register_user))
}

#[derive(Deserialize, Validate)]
struct RegisterUserPayload {
    #[validate(regex = "USER_REGEX")]
    username: String,
    #[validate(length(min = 8))]
    password: String,
}

#[derive(Serialize)]
struct RegisterUserBody {
    username: String,
}

async fn register_user(
    ctx: Extension<ApiContext>,
    OriginalUri(uri): OriginalUri,
    Json(payload): Json<RegisterUserPayload>,
) -> Result<CreatedJson<RegisterUserBody>, ApiError> {
    if let Err(e) = payload.validate() {
        if e.errors().contains_key("username") {
            return Err(ApiError::BadRequest("invalid user name".to_string()));
        }
        if e.errors().contains_key("password") {
            return Err(ApiError::BadRequest(
                "password must be at least 8 characters".to_string(),
            ));
        }
    }
    let username = payload.username;
    let password = payload.password;
    let hash = Scrypt
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .context("failed to hash password")?
        .to_string();
    let result = query!(
        "INSERT INTO users (user_id, pw_hash) VALUES ($1, $2)",
        username,
        hash
    )
    .execute(&ctx.db)
    .await
    .map_err(|error| match error {
        sqlx::Error::Database(db_err) if db_err.code().unwrap_or_default() == "23505" => {
            ApiError::Conflict("user name already exists".to_string())
        }
        _ => ApiError::ServerError(anyhow!("failed to create user")),
    })?;
    match result.rows_affected() {
        1 => {
            let uri = format!("{}/{}", uri, username);
            let body = RegisterUserBody { username };
            Ok(CreatedJson(uri, body))
        }
        _ => Err(ApiError::ServerError(anyhow!("failed to create user"))),
    }
}
