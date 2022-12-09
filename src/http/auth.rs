use axum::{routing::post, Extension, Router};
use sqlx::PgPool;

use serde::Deserialize;
use thiserror::Error;

use crate::{
    http::{
        self, json,
        session::{self, Session},
    },
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
    session_store: Extension<session::Store>,
    json::extractor::Json(req): json::extractor::Json<CreateAuthSession>,
) -> Result<(http::HeaderMap, http::StatusCode), http::Error>
{
    let CreateAuthSession { username, password } = req;

    let user = sqlx::query!(
        r#"select user_id, password from users where username = $1"#,
        username
    )
    .fetch_optional(&*pg_pool)
    .await?;

    match user {
        Some(user) => {
            let password_is_correct = password::verify(password, user.password).await?;

            if password_is_correct {
                let mut session = Session::new();
                session.insert("user_id", user.user_id).await?;
                // SAFETY: This cannot fail as store_session propagates `None`
                // upon a `None` field for the session's cookie value, which
                // will never be empty as we create the session above and never
                // mutate its cookie value
                let cookie = session_store.store_session(session).await?.unwrap();

                let mut headers = http::HeaderMap::new();
                let header_value = http::HeaderValue::from_str(&format!(
                    "{}={}",
                    session::SESSION_COOKIE_NAME,
                    cookie
                ))
                // SAFETY: It is known in advance that `SESSION_COOKIE_NAME` as
                // well as the cookie propagated from the init of the session
                // are both always going to be ASCII-only, therefore the
                // formatted string will be ASCII-only, so creating the
                // HeaderValue will never fail
                .unwrap();
                let _prev_value = headers.insert(http::header::SET_COOKIE, header_value);

                Ok((headers, http::StatusCode::NO_CONTENT))
            } else {
                Err(Error::WrongPassword)?
            }
        }
        None => Err(Error::UserNotFound)?,
    }
}

#[derive(Debug, Error)]
enum Error
{
    #[error("no user with the provided was found")]
    UserNotFound,
    #[error("the provided password is wrong")]
    WrongPassword,
}

impl From<Error> for http::Error
{
    fn from(err: Error) -> Self
    {
        let error_code = match err {
            Error::UserNotFound => http::error::Code::USER_NOT_FOUND,
            Error::WrongPassword => http::error::Code::WRONG_PASSWORD,
        };

        let status_code = match err {
            Error::UserNotFound => http::StatusCode::NOT_FOUND,
            Error::WrongPassword => http::StatusCode::UNPROCESSABLE_ENTITY,
        };

        let message = err.to_string();

        http::Error {
            error_code,
            status_code,
            message,
        }
    }
}
