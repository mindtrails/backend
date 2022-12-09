use std::net::SocketAddr;

use axum::{Extension, Router, Server};
use sqlx::PgPool;

mod error;
pub(in crate::http) use error::Error;

mod json;

mod auth;
mod users;

pub type StatusCode = ::axum::http::StatusCode;

fn app(pg_pool: PgPool) -> Router
{
    Router::new()
        .merge(auth::router())
        .merge(users::router())
        .layer(Extension(pg_pool))
}

pub async fn serve(port: u16, pg_pool: PgPool) -> Result<(), hyper::Error>
{
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    Server::bind(&addr)
        .serve(app(pg_pool).into_make_service())
        .await?;

    Ok(())
}
