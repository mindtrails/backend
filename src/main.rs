use std::{net::SocketAddr, time::Duration};

use axum::{extract::State, http::status::StatusCode, routing::get, Router, Server};
use sqlx::{self, postgres::PgPoolOptions, PgPool};

use thiserror::Error;

use mindtrails_backend::config::{self, Config};

#[tokio::main]
async fn main() -> Result<(), self::Error>
{
    let config = Config::init()?;

    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(config.postgres_url())
        .await?;

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port()));

    let app = Router::new().route("/", get(root)).with_state(pg_pool);

    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

async fn root(State(pg_pool): State<PgPool>) -> Result<String, (StatusCode, String)>
{
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(256_i64)
        .fetch_one(&pg_pool)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(row.0.to_string())
}

#[derive(Debug, Error)]
enum Error
{
    #[error("{0}")]
    Config(#[from] config::Error),
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("{0}")]
    Hyper(#[from] hyper::Error),
}
