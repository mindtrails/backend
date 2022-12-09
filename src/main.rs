use std::time::Duration;

use sqlx::{self, postgres::PgPoolOptions};

use thiserror::Error;

use mindtrails::{
    config::{self, Config},
    http,
};

#[tokio::main]
async fn main() -> Result<(), self::Error>
{
    let config = Config::init()?;

    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(config.postgres_url())
        .await?;
    sqlx::migrate!().run(&pg_pool).await?;

    http::serve(config.port(), pg_pool).await?;

    Ok(())
}

#[derive(Debug, Error)]
enum Error
{
    #[error("{0}")]
    Config(#[from] config::Error),
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("{0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("{0}")]
    Http(#[from] http::Error),
}
