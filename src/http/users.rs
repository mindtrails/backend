use axum::{routing::post, Extension, Router};
use sqlx::PgPool;

use serde::Deserialize;
use thiserror::Error;

use crate::{
    http::{self, json},
    password,
};

pub(in crate::http) fn router() -> Router
{
    Router::new().route("/users", post(create_user))
}

#[derive(Deserialize)]
struct CreateUser
{
    username: String,
    password: String,
}

async fn create_user(
    pg_pool: Extension<PgPool>,
    json::extractor::Json(req): json::extractor::Json<CreateUser>,
) -> Result<http::StatusCode, http::Error>
{
    let CreateUser { username, password } = req;

    let password = password::hash(password).await?;

    let pg_query_res = sqlx::query!(
        r#"
            INSERT INTO "users"(username, password)
            values ($1, $2)
        "#,
        username,
        password
    )
    .execute(&*pg_pool)
    .await;

    match pg_query_res {
        Ok(_) => Ok(http::StatusCode::NO_CONTENT),
        Err(sqlx::Error::Database(database_err))
            if database_err.constraint() == Some("users_username_key") =>
        {
            Err(Error::UsernameTaken)?
        }
        Err(err) => Err(err)?,
    }
}

#[derive(Debug, Error)]
enum Error
{
    #[error("username already taken")]
    UsernameTaken,
}

impl From<Error> for http::Error
{
    fn from(err: Error) -> Self
    {
        let error_code = match err {
            Error::UsernameTaken => http::error::Code::USERNAME_TAKEN,
        };

        let status_code = match err {
            Error::UsernameTaken => http::StatusCode::CONFLICT,
        };

        let message = err.to_string();

        http::Error {
            error_code,
            status_code,
            message,
        }
    }
}
