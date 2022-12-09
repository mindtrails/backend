pub(in crate::http) mod extractor
{
    use crate::http::{self, error};

    use axum::{extract::rejection, response};

    use axum_macros::FromRequest;
    use serde_json::json;

    #[derive(FromRequest)]
    #[from_request(via(axum::Json), rejection(Error))]
    pub(in crate::http) struct Json<T>(pub(in crate::http) T);

    #[derive(Debug)]
    pub(in crate::http) struct Error
    {
        error_code: error::Code,
        status_code: http::StatusCode,
        message: String,
    }

    impl From<rejection::JsonRejection> for Error
    {
        fn from(rejection: rejection::JsonRejection) -> Self
        {
            let (error_code, status_code) = match rejection {
                rejection::JsonRejection::JsonSyntaxError(_) => (
                    error::Code::JSON_SYNTAX_ERROR,
                    http::StatusCode::BAD_REQUEST,
                ),
                rejection::JsonRejection::JsonDataError(_) => (
                    error::Code::JSON_DATA_ERROR,
                    http::StatusCode::UNPROCESSABLE_ENTITY,
                ),
                rejection::JsonRejection::MissingJsonContentType(_) => (
                    error::Code::JSON_MISSING_CONTENT_TYPE,
                    http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
                ),
                _ => (
                    error::Code::JSON_UNKNOWN_ERROR,
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                ),
            };

            let message = rejection.to_string();

            Error {
                error_code,
                status_code,
                message,
            }
        }
    }

    impl response::IntoResponse for Error
    {
        fn into_response(self) -> response::Response
        {
            let payload = json!({
                "message": self.message,
                "code": self.error_code
            });

            (self.status_code, axum::Json(payload)).into_response()
        }
    }
}
