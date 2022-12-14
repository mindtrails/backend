use std::net::SocketAddr;

use axum::{Extension, Router, Server};
use sqlx::PgPool;
use tokio::signal;
use tower_http::cors::CorsLayer;

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

fn app(cors: CorsLayer, pg_pool: PgPool, session_store: session::Store) -> Router
{
    Router::new()
        .merge(auth::router())
        .merge(users::router())
        .layer(Extension(pg_pool))
        .layer(Extension(session_store))
        .layer(cors)
}

pub async fn serve(
    in_production: bool,
    port: u16,
    pg_pool: PgPool,
    session_store: session::Store,
) -> Result<(), hyper::Error>
{
    let addr = if in_production {
        SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], port))
    } else {
        SocketAddr::from(([127, 0, 0, 1], port))
    };

    let cors_origin = if in_production {
        ::std::env::var("CORS_ORIGIN").unwrap()
    } else {
        format!("http://127.0.0.1:3000")
    }
    .parse::<HeaderValue>()
    .unwrap();
    let cors = CorsLayer::new()
        .allow_methods([
            ::axum::http::Method::GET,
            ::axum::http::Method::POST,
            ::axum::http::Method::DELETE,
        ])
        .allow_credentials(true)
        .allow_origin(cors_origin)
        .allow_headers([::axum::http::header::CONTENT_TYPE]);

    Server::bind(&addr)
        .serve(app(cors, pg_pool, session_store).into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal()
{
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // NOTE: I don't run a Unix machine so I don't actually know if this works
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {}
    }
}
