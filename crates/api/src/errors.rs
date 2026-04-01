use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tracing::error;

#[derive(Debug)]
pub(crate) enum ApiError {
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            Self::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
        };

        error!(status = %status, error = %msg, "Request failed");
        (status, Json(ErrorBody { error: msg })).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_error() {
        let error = ApiError::Internal("Internal error".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
