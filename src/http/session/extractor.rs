use axum::{
    extract::FromRequestParts, headers::Cookie, http::request, Extension, RequestPartsExt,
    TypedHeader,
};

use async_trait::async_trait;
use uuid::Uuid;

use crate::http::{self, session};

#[derive(Debug)]
pub(in crate::http) enum UserId
{
    Found(Uuid),
    NotFound,
}

#[async_trait]
impl<S> FromRequestParts<S> for UserId
where
    S: Send + Sync,
{
    type Rejection = http::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection>
    {
        let store = parts
            .extract::<Extension<session::Store>>()
            .await
            .map_err(|_| session::Error::MissingStoreExtension)?;
        let cookie = parts
            .extract::<Option<TypedHeader<Cookie>>>()
            .await
            // SAFETY: Unwrapping `Result<T, Infallible>` is guaranteed to
            // never panic
            .unwrap();
        let session_cookie = cookie
            .as_ref()
            .and_then(|cookie| cookie.get(session::SESSION_COOKIE_NAME));

        match session_cookie {
            Some(session_cookie) => {
                let session = store.load_session(session_cookie).await?;

                if let Some(user_id) = session.get::<Uuid>("user_id").await {
                    Ok(UserId::Found(user_id))
                } else {
                    Ok(UserId::NotFound)
                }
            }
            None => Ok(UserId::NotFound),
        }
    }
}

#[derive(Debug)]
pub(in crate::http) enum Session
{
    Found(session::Session),
    NotFound,
}

#[async_trait]
impl<S> FromRequestParts<S> for Session
where
    S: Send + Sync,
{
    type Rejection = http::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection>
    {
        let store = parts
            .extract::<Extension<session::Store>>()
            .await
            .map_err(|_| session::Error::MissingStoreExtension)?;
        let cookie = parts
            .extract::<Option<TypedHeader<Cookie>>>()
            .await
            // SAFETY: Unwrapping `Result<T, Infallible>` is guaranteed to
            // never panic
            .unwrap();
        let session_cookie = cookie
            .as_ref()
            .and_then(|cookie| cookie.get(session::SESSION_COOKIE_NAME));

        match session_cookie {
            Some(session_cookie) => {
                let session = store.load_session(session_cookie).await?;

                Ok(Session::Found(session))
            }
            None => Ok(Session::NotFound),
        }
    }
}
