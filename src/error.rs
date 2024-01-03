use std::borrow::Cow;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum AuthError {
    /// The 'Authorization' header was not present on a request.
    #[snafu(display("The 'Authorization' header was not present on a request."))]
    MissingAuthorizationHeader,

    /// The 'Authorization' header was present on a request but its value could not be parsed.
    /// This can occur if the header value did not solely contain visible ASCII characters.
    #[snafu(display("The 'Authorization' header was present on a request but its value could not be parsed. Reason: {reason}"))]
    InvalidAuthorizationHeader { reason: String },

    /// The 'Authorization' header was present  and could be parsed, but it did not contain the expected "Bearer {token}" format.
    #[snafu(display(
        "The 'Authorization' header did not contain the expected 'Bearer ...token' format."
    ))]
    MissingBearerToken,

    /// The DecodingKey, required for decoding tokens, could not be created.
    #[snafu(display(
        "The DecodingKey, required for decoding tokens, could not be created. Source: {source}"
    ))]
    CreateDecodingKey { source: jsonwebtoken::errors::Error },

    /// The JWT header could not be decoded.
    #[snafu(display("The JWT header could not be decoded. Source: {source}"))]
    DecodeHeader { source: jsonwebtoken::errors::Error },

    /// The JWT could not be decoded.
    #[snafu(display("The JWT could not be decoded. Source: {source}"))]
    Decode { source: jsonwebtoken::errors::Error },

    /// Parts of the JWT could not be parsed.
    #[snafu(display("Parts of the JWT could not be parsed. Source: {source}"))]
    JsonParse { source: serde_json::Error },

    /// The tokens lifetime is expired.
    #[snafu(display("The tokens lifetime is expired."))]
    TokenExpired,

    /// For a not further known reason, the token was deemed invalid
    #[snafu(display(
        "For a not further known reason, the token was deemed invalid: Reason: {reason}"
    ))]
    InvalidToken { reason: String },

    /// Note: The `IntoResponse` implementation will only show the provided role in a debug build!
    #[snafu(display("An expected role (omitted for security reasons) was missing."))]
    MissingExpectedRole { role: String },

    /// An unexpected role was present.
    #[snafu(display("An unexpected role was present."))]
    UnexpectedRole,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            err @ AuthError::MissingAuthorizationHeader => {
                (StatusCode::BAD_REQUEST, Cow::Owned(err.to_string()))
            }
            err @ AuthError::InvalidAuthorizationHeader { reason: _ } => {
                (StatusCode::BAD_REQUEST, Cow::Owned(err.to_string()))
            }
            err @ AuthError::MissingBearerToken => {
                (StatusCode::BAD_REQUEST, Cow::Owned(err.to_string()))
            }
            err @ AuthError::CreateDecodingKey { source: _ } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Cow::Owned(err.to_string()),
            ),
            err @ AuthError::DecodeHeader { source: _ } => {
                (StatusCode::UNAUTHORIZED, Cow::Owned(err.to_string()))
            }
            err @ AuthError::Decode { source: _ } => {
                (StatusCode::UNAUTHORIZED, Cow::Owned(err.to_string()))
            }
            err @ AuthError::JsonParse { source: _ } => {
                (StatusCode::UNAUTHORIZED, Cow::Owned(err.to_string()))
            }
            err @ AuthError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, Cow::Owned(err.to_string()))
            }
            err @ AuthError::InvalidToken { reason: _ } => {
                (StatusCode::BAD_REQUEST, Cow::Owned(err.to_string()))
            }
            AuthError::MissingExpectedRole { role } => (
                StatusCode::UNAUTHORIZED,
                match cfg!(debug_assertions) {
                    true => Cow::Owned(format!("Missing expected role: {role}")),
                    false => Cow::Borrowed("Missing expected role"),
                },
            ),
            err @ AuthError::UnexpectedRole => {
                (StatusCode::UNAUTHORIZED, Cow::Owned(err.to_string()))
            }
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
