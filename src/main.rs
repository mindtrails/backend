#[tokio::main]
async fn main()
{
    let app = axum::Router::new().route("/", axum::routing::get(root));

    let addr = ::std::net::SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

async fn root() -> &'static str
{
    "Hello, world!"
}
