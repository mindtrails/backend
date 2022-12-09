pub(in crate::http) mod extractor
{
    use crate::http::{self, error};

    use axum::extract::rejection;

    use axum_macros::FromRequest;

    #[derive(FromRequest)]
    #[from_request(via(axum::Json), rejection(http::Error))]
    pub(in crate::http) struct Json<T>(pub(in crate::http) T);

    impl From<rejection::JsonRejection> for http::Error
    {
        fn from(rejection: rejection::JsonRejection) -> Self
        {
            let error_code = match rejection {
                rejection::JsonRejection::JsonSyntaxError(_) => error::Code::JSON_SYNTAX_ERROR,
                rejection::JsonRejection::JsonDataError(_) => error::Code::JSON_DATA_ERROR,
                rejection::JsonRejection::MissingJsonContentType(_) => {
                    error::Code::JSON_MISSING_CONTENT_TYPE
                }
                _ => error::Code::JSON_UNKNOWN_ERROR,
            };

            let status_code = match rejection {
                rejection::JsonRejection::JsonSyntaxError(_) => http::StatusCode::BAD_REQUEST,
                rejection::JsonRejection::JsonDataError(_) => {
                    http::StatusCode::UNPROCESSABLE_ENTITY
                }
                rejection::JsonRejection::MissingJsonContentType(_) => {
                    http::StatusCode::UNSUPPORTED_MEDIA_TYPE
                }
                _ => http::StatusCode::INTERNAL_SERVER_ERROR,
            };

            let message = match rejection {
                rejection::JsonRejection::JsonSyntaxError(_)
                | rejection::JsonRejection::JsonDataError(_)
                | rejection::JsonRejection::MissingJsonContentType(_) => rejection.to_string(),
                _ => String::from(http::error::INTERNAL_SERVER_ERROR_MESSAGE),
            };

            http::Error {
                error_code,
                status_code,
                message,
            }
        }
    }
}
