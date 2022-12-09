use thiserror::Error;

#[tokio::main]
async fn main() -> Result<(), self::Error>
{
    let config = mindtrails_backend::config::Config::init()?;

    let app = axum::Router::new().route("/", axum::routing::get(root));

    let addr = ::std::net::SocketAddr::from(([127, 0, 0, 1], config.port()));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn root() -> &'static str
{
    "Hello, world!"
}

use mindtrails_backend::config;

#[derive(Debug, Error)]
enum Error
{
    #[error("{0}")]
    Config(#[from] config::Error),
    #[error("{0}")]
    Hyper(#[from] hyper::Error),
}
