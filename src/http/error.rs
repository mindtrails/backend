use std::num::NonZeroU16;

use axum::{response, Json};

use serde::Serialize;
use serde_json::json;

use crate::http;

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

// 000 - Internal Server Error
// 1xx - JSON
//     100 - JSON Syntax Error
//     110 - JSON Data Error
//     120 - JSON Missing Content Type
//     199 - JSON Unknown Error
// 5xx - Users
//     501 - Username Taken
impl Code
{
    #![allow(unsafe_code)]

    code!(INTERNAL_SERVER_ERROR, 000);

    code!(JSON_SYNTAX_ERROR, 100);
    code!(JSON_DATA_ERROR, 110);
    code!(JSON_MISSING_CONTENT_TYPE, 120);
    code!(JSON_UNKNOWN_ERROR, 199);

    code!(USERNAME_TAKEN, 501);
}
