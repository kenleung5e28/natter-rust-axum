use crate::api::{ApiContext, ApiError};
use axum::{
    extract::{FromRequest, RequestParts, TypedHeader},
    headers::ContentType,
    http::{Method, Request},
    middleware::Next,
    response::Response,
    Extension,
};

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
            if let Err(_) = ctx.limiter.check() {
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
