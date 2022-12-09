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
    Router::new().route("/auth", post(create_auth_session))
}

#[derive(Deserialize)]
struct CreateAuthSession
{
    username: String,
    password: String,
}

async fn create_auth_session(
    pg_pool: Extension<PgPool>,
    json::extractor::Json(req): json::extractor::Json<CreateAuthSession>,
) -> Result<http::StatusCode, http::Error>
{
    let CreateAuthSession { username, password } = req;

    let user = sqlx::query!(
        r#"select user_id, password from users where username = $1"#,
        username
    )
    .fetch_optional(&*pg_pool)
    .await
    .map_err(Error::from)?;

    match user {
        Some(user) => {
            let password_is_correct = password::verify(password, user.password).await?;

            if password_is_correct {
                Ok(http::StatusCode::NO_CONTENT)
            } else {
                Err(Error::WrongPassword)?
            }
        }
        None => Err(Error::UserNotFound { username })?,
    }
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
    #[error("no user with username {username} was found")]
    UserNotFound
    {
        username: String
    },
    #[error("the provided password is wrong")]
    WrongPassword,
}

impl From<Error> for http::Error
{
    fn from(err: Error) -> Self
    {
        let error_code = match err {
            Error::Sqlx { .. } => http::error::Code::INTERNAL_SERVER_ERROR,
            Error::UserNotFound { .. } => http::error::Code::USER_NOT_FOUND,
            Error::WrongPassword => http::error::Code::WRONG_PASSWORD,
        };

        let status_code = match err {
            Error::Sqlx { .. } => http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::UserNotFound { .. } => http::StatusCode::NOT_FOUND,
            Error::WrongPassword => http::StatusCode::UNPROCESSABLE_ENTITY,
        };

        let message = match err {
            Error::Sqlx { .. } => String::from(http::error::INTERNAL_SERVER_ERROR_MESSAGE),
            err => err.to_string(),
        };

        http::Error {
            error_code,
            status_code,
            message,
        }
    }
}
