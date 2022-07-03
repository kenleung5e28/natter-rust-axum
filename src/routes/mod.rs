use sqlx::PgPool;

pub mod space;

#[derive(Clone)]
pub struct ApiContext {
    pub db: PgPool,
}
