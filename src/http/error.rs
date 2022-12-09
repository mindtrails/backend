use std::num::NonZeroU16;

use axum::{response, Json};

use serde::Serialize;
use serde_json::json;

use crate::{http, password};

#[derive(Debug)]
pub(in crate::http) struct Error
{
    pub(in crate::http) error_code: http::error::Code,
    pub(in crate::http) status_code: http::StatusCode,
    pub(in crate::http) message: String,
}

impl response::IntoResponse for Error
{
    fn into_response(self) -> response::Response
    {
        let payload = json!({
            "message": self.message,
            "code": self.error_code
        });

        (self.status_code, Json(payload)).into_response()
    }
}

#[derive(Debug, Serialize)]
pub(super) struct Code(NonZeroU16);

macro_rules! code {
    ($name:ident, $code:expr) => {
        pub(super) const $name: Code =
            Code(unsafe { ::std::num::NonZeroU16::new_unchecked($code) });
    };
}

pub(super) const INTERNAL_SERVER_ERROR_MESSAGE: &str = "Internal Server Error";

// 1xx - JSON
//     100 - JSON Syntax Error
//     110 - JSON Data Error
//     120 - JSON Missing Content Type
//     199 - JSON Unknown Error
// 4xx - Auth
//     401 - User Not Found
//     402 - Wrong Password
// 5xx - Users
//     501 - Username Taken
// 999 - Internal Server Error
impl Code
{
    #![allow(unsafe_code)]

    code!(JSON_SYNTAX_ERROR, 100);
    code!(JSON_DATA_ERROR, 110);
    code!(JSON_MISSING_CONTENT_TYPE, 120);

    code!(USER_NOT_FOUND, 401);
    code!(WRONG_PASSWORD, 402);

    code!(USERNAME_TAKEN, 501);

    code!(INTERNAL_SERVER_ERROR, 999);
}

impl From<sqlx::Error> for Error
{
    fn from(_sqlx_err: sqlx::Error) -> Self
    {
        Error {
            error_code: Code::INTERNAL_SERVER_ERROR,
            status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            message: String::from(INTERNAL_SERVER_ERROR_MESSAGE),
        }
    }
}

impl From<password::Error> for Error
{
    fn from(_password_err: password::Error) -> Self
    {
        Error {
            error_code: Code::INTERNAL_SERVER_ERROR,
            status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            message: String::from(INTERNAL_SERVER_ERROR_MESSAGE),
        }
    }
}
