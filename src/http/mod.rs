use std::net::SocketAddr;

use axum::{Extension, Router, Server};
use sqlx::PgPool;

mod error;
pub(in crate::http) use error::Error;

mod json;
pub mod session;

mod auth;
mod users;

type StatusCode = ::axum::http::StatusCode;

use axum::http::header;
type HeaderMap = ::axum::http::HeaderMap;
type HeaderValue = ::axum::http::HeaderValue;

fn app(pg_pool: PgPool, session_store: session::Store) -> Router
{
    Router::new()
        .merge(auth::router())
        .merge(users::router())
        .layer(Extension(pg_pool))
        .layer(Extension(session_store))
}

pub async fn serve(
    port: u16,
    pg_pool: PgPool,
    session_store: session::Store,
) -> Result<(), hyper::Error>
{
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    Server::bind(&addr)
        .serve(app(pg_pool, session_store).into_make_service())
        .await?;

    Ok(())
}
