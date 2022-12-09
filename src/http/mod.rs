use std::net::SocketAddr;

use axum::{http::StatusCode, routing::get, Extension, Router, Server};
use sqlx::PgPool;

use thiserror::Error;

mod json;

fn root() -> Router
{
    Router::new().route(
        "/",
        get(
            async move |pg_pool: Extension<PgPool>| -> Result<String, (StatusCode, String)> {
                let row: (i64,) = sqlx::query_as("SELECT $1")
                    .bind(256_i64)
                    .fetch_one(&*pg_pool)
                    .await
                    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

                Ok(row.0.to_string())
            },
        ),
    )
}

fn app(pg_pool: PgPool) -> Router
{
    Router::new().merge(root()).layer(Extension(pg_pool))
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
    #[error("{0}")]
    Hyper(#[from] hyper::Error),
}

mod error
{
    use std::num::NonZeroU16;

    use serde::Serialize;

    macro_rules! code {
        ($name:ident, $code:expr) => {
            pub(super) const $name: Code =
                Code(unsafe { ::std::num::NonZeroU16::new_unchecked($code) });
        };
    }

    #[derive(Debug, Serialize)]
    pub(super) struct Code(NonZeroU16);

    // 1xx - JSON
    //     100 - JSON Syntax Error
    //     110 - JSON Data Error
    //     120 - JSON Missing Content Type
    //     199 - JSON Unknown Error
    impl Code
    {
        #![allow(unsafe_code)]

        code!(JSON_SYNTAX_ERROR, 100);
        code!(JSON_DATA_ERROR, 110);
        code!(JSON_MISSING_CONTENT_TYPE, 120);
        code!(JSON_UNKNOWN_ERROR, 199);
    }
}
