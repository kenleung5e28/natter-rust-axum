use crate::api::{ApiContext, AuthContext};
use crate::error::ApiError;
use crate::routes::USER_REGEX;
use anyhow::anyhow;
use axum::{
    extract::{FromRequest, RequestParts, TypedHeader},
    headers::{authorization, Authorization, ContentType},
    http::{Method, Request},
    middleware::Next,
    response::Response,
    Extension,
};
use scrypt::password_hash::PasswordVerifier;
use scrypt::{password_hash::PasswordHash, Scrypt};
use sqlx::{query, query_scalar};

pub async fn accept_only_json_payload_in_post<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError>
where
    B: Send,
{
    if req.method() != Method::POST {
        return Ok(next.run(req).await);
    }
    let mut req_parts = RequestParts::<B>::new(req);
    match TypedHeader::<ContentType>::from_request(&mut req_parts).await {
        Ok(TypedHeader(content_type)) => {
            if content_type != ContentType::json() {
                return Err(ApiError::OnlySupportJsonContentType);
            }
        }
        Err(rejection) => return Err(ApiError::ServerError(rejection.into())),
    }
    let req = req_parts
        .try_into_request()
        .expect("body should not be extracted");
    Ok(next.run(req).await)
}

pub async fn rate_limit_requests<B>(req: Request<B>, next: Next<B>) -> Result<Response, ApiError>
where
    B: Send,
{
    let mut req_parts = RequestParts::<B>::new(req);
    match Extension::<ApiContext>::from_request(&mut req_parts).await {
        Ok(ctx) => {
            if ctx.limiter.check().is_err() {
                return Err(ApiError::TooManyRequests);
            }
        }
        Err(rejection) => return Err(ApiError::ServerError(rejection.into())),
    }
    let req = req_parts
        .try_into_request()
        .expect("body should not be extracted");
    Ok(next.run(req).await)
}

pub async fn authenticate<B>(req: Request<B>, next: Next<B>) -> Result<Response, ApiError>
where
    B: Send,
{
    let mut auth_ctx = AuthContext { subject: None };
    let mut req_parts = RequestParts::<B>::new(req);
    if let Ok(TypedHeader(basic_auth)) =
        TypedHeader::<Authorization<authorization::Basic>>::from_request(&mut req_parts).await
    {
        let username = basic_auth.username();
        let password = basic_auth.password();
        if !USER_REGEX.is_match(username) {
            return Err(ApiError::BadRequest("invalid user name".to_string()));
        }
        let ctx = req_parts
            .extensions()
            .get::<ApiContext>()
            .ok_or_else(|| ApiError::ServerError(anyhow!("failed to fetch context")))?;
        let result = query_scalar!("SELECT pw_hash FROM users WHERE user_id = $1", username)
            .fetch_optional(&ctx.db)
            .await?;
        if let Some(hash) = result {
            if let Ok(parsed_hash) = PasswordHash::new(&hash) {
                if Scrypt
                    .verify_password(password.as_bytes(), &parsed_hash)
                    .is_ok()
                {
                    auth_ctx = AuthContext {
                        subject: Some(username.to_string()),
                    };
                }
            }
        }
    }
    req_parts.extensions_mut().insert(auth_ctx);
    let req = req_parts
        .try_into_request()
        .expect("body should not be extracted");
    Ok(next.run(req).await)
}

pub async fn audit_request<B>(req: Request<B>, next: Next<B>) -> Result<Response, ApiError>
where
    B: Send,
{
    let mut req_parts = RequestParts::<B>::new(req);
    let ctx = Extension::<ApiContext>::from_request(&mut req_parts)
        .await
        .map_err(|rejection| ApiError::ServerError(rejection.into()))?;
    let mut transaction = ctx.db.begin().await?;
    let audit_id = query_scalar!("SELECT nextval('audit_id_seq')")
        .fetch_one(&mut transaction)
        .await?;
    if audit_id.is_none() {
        transaction.rollback().await?;
        return Err(ApiError::ServerError(anyhow!(
            "failed to obtain next audit id"
        )));
    }
    let audit_id = audit_id.unwrap();
    let user_id = Extension::<AuthContext>::from_request(&mut req_parts)
        .await
        .ok()
        .and_then(|ctx| ctx.subject.as_deref().map(|s| s.to_string()));
    let request_method = String::from(req_parts.method().as_str());
    let request_path = String::from(req_parts.uri().path());
    query!(
        "INSERT INTO audit_log(audit_id, method, path, user_id) VALUES ($1, $2, $3, $4)",
        audit_id,
        request_method,
        request_path,
        user_id
    )
    .execute(&mut transaction)
    .await?;
    let req = req_parts
        .try_into_request()
        .expect("body should not be extracted");
    transaction.commit().await?;
    let res = next.run(req).await;
    let response_status = i32::from(res.status().as_u16());
    query!(
        "INSERT INTO audit_log(audit_id, method, path, status, user_id) VALUES ($1, $2, $3, $4, $5)",
        audit_id,
        request_method,
        request_path,
        response_status,
        user_id
    )
    .execute(&ctx.db)
    .await?;
    Ok(res)
}
