use std::net::SocketAddr;

use axum::{Extension, Router, Server};
use sqlx::PgPool;

use thiserror::Error;

mod error;
mod json;

mod users;

pub type StatusCode = ::axum::http::StatusCode;

fn app(pg_pool: PgPool) -> Router
{
    Router::new()
        .merge(users::router())
        .layer(Extension(pg_pool))
}

pub async fn serve(port: u16, pg_pool: PgPool) -> Result<(), self::Error>
{
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    Server::bind(&addr)
        .serve(app(pg_pool).into_make_service())
        .await?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum Error
{
    #[error("{inner}")]
    Hyper
    {
        #[from]
        inner: hyper::Error,
    },
}
