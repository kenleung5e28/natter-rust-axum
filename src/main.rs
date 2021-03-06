use anyhow::Context;
use axum::{middleware::from_fn, Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use governor::{Quota, RateLimiter};
use http::header::{
    HeaderValue, CACHE_CONTROL, CONTENT_SECURITY_POLICY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
    X_XSS_PROTECTION,
};
use nonzero_ext::nonzero;
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, num::NonZeroU32, path::PathBuf, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{set_header::SetResponseHeaderLayer, trace::TraceLayer};

mod api;
mod error;
mod middlewares;
mod routes;

const DEFAULT_RATE_LIMIT: NonZeroU32 = nonzero!(2u32);

#[derive(Debug, Parser)]
struct Config {
    #[clap(long, env)]
    app_database_url: String,
    #[clap(long, env, default_value_t = DEFAULT_RATE_LIMIT)]
    rate_limit: NonZeroU32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let config = Config::parse();

    tracing_subscriber::fmt::init();

    let cert_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("self-signed-certs");
    let tls_config = RustlsConfig::from_pem_file(
        cert_path.join("localhost.pem"),
        cert_path.join("localhost-key.pem"),
    )
    .await
    .context("failed to configure TLS certificates")?;

    let db = PgPoolOptions::new()
        .max_connections(100)
        .connect(&config.app_database_url)
        .await
        .context("unable to connect to database")?;

    let limiter = Arc::new(RateLimiter::direct(Quota::per_second(DEFAULT_RATE_LIMIT)));

    let app = Router::new()
        .nest(
            "/spaces",
            routes::space::router().merge(routes::moderator::router()),
        )
        .nest("/users", routes::user::router())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(Extension(api::ApiContext { db, limiter }))
                .layer(SetResponseHeaderLayer::overriding(
                    X_CONTENT_TYPE_OPTIONS,
                    HeaderValue::from_static("nosniff"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    X_FRAME_OPTIONS,
                    HeaderValue::from_static("DENY"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    X_XSS_PROTECTION,
                    HeaderValue::from_static("0"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    CACHE_CONTROL,
                    HeaderValue::from_static("no-store"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    CONTENT_SECURITY_POLICY,
                    HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'; sandbox"),
                ))
                .layer(from_fn(middlewares::accept_only_json_payload_in_post))
                .layer(from_fn(middlewares::rate_limit_requests))
                .layer(from_fn(middlewares::authenticate))
                .layer(from_fn(middlewares::audit_request)),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::debug!("listening on {}", addr);
    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .context("error running HTTP server")
}
