use thiserror::Error;

#[tokio::main]
async fn main() -> Result<(), self::Error>
{
    let app = axum::Router::new().route("/", axum::routing::get(root));

    let addr = ::std::net::SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn root() -> &'static str
{
    "Hello, world!"
}

#[derive(Debug, Error)]
enum Error
{
    #[error("{0}")]
    Hyper(#[from] hyper::Error),
}
