use anyhow::Context;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;

#[derive(clap::Parser)]
struct Config {
    #[clap(long, env)]
    connection_string: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let config = Config::parse();

    tracing_subscriber::fmt::init();

    let db = PgPoolOptions::new()
        .max_connections(100)
        .connect(&config.connection_string)
        .await
        .context("unable to connect to database")?;

    sqlx::migrate!().run(&db).await?;

    let app = Router::new();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("error running HTTP server")
}
