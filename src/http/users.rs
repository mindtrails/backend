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
#[serde(rename_all = "camelCase")]
struct CreateUser
{
    username: String,
    password: String,
}

async fn create_user(
    pg_pool: Extension<PgPool>,
    json::extractor::Json(req): json::extractor::Json<CreateUser>,
) -> Result<http::StatusCode, http::error::Error>
{
    let CreateUser { username, password } = req;

    let password = password::hash(password).await?;

    let _pg_query_res = sqlx::query!(
        r#"
            INSERT INTO "users"(username, password)
            values ($1, $2)
        "#,
        username,
        password
    )
    .execute(&*pg_pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::Database(database_err)
            if database_err.constraint() == Some("users_username_key") =>
        {
            Error::UsernameTaken
        }
        err => Error::from(err),
    })?;

    Ok(http::StatusCode::NO_CONTENT)
}

#[derive(Debug, Error)]
enum Error
{
    #[error("{inner}")]
    Sqlx
    {
        #[from]
        inner: sqlx::Error,
    },
    #[error("username already taken")]
    UsernameTaken,
}

impl From<Error> for http::error::Error
{
    fn from(err: Error) -> Self
    {
        let error_code = match err {
            Error::Sqlx { .. } => http::error::Code::INTERNAL_SERVER_ERROR,
            Error::UsernameTaken => http::error::Code::USERNAME_TAKEN,
        };

        let status_code = match err {
            Error::Sqlx { .. } => http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::UsernameTaken => http::StatusCode::CONFLICT,
        };

        let message = match err {
            Error::Sqlx { .. } => String::from(http::error::INTERNAL_SERVER_ERROR_MESSAGE),
            err => err.to_string(),
        };

        http::error::Error {
            error_code,
            status_code,
            message,
        }
    }
}
