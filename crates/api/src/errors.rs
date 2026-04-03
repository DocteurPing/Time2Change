use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tracing::error;

#[derive(Debug)]
pub(crate) enum ApiError {
    Internal(String),
    InvalidCurrency(String),
    InvalidCurrencyPair(String),
    NotEnoughData(String),
    InvalidDaysLookBack(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            Self::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
            Self::NotEnoughData(m) => (StatusCode::UNPROCESSABLE_ENTITY, m),
            Self::InvalidCurrency(m)
            | Self::InvalidCurrencyPair(m)
            | Self::InvalidDaysLookBack(m) => (StatusCode::BAD_REQUEST, m),
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
        let error = ApiError::Internal("Internal error".to_owned());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_invalid_currency() {
        let error = ApiError::InvalidCurrency("Currency is invalid".to_owned());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_invalid_currency_pair() {
        let error = ApiError::InvalidCurrencyPair("Currencies are the same".to_owned());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_not_enough_data() {
        let error = ApiError::NotEnoughData("Not enough data".to_owned());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn test_invalid_days_look_back() {
        let error = ApiError::InvalidDaysLookBack("Days must be between 1 and 365".to_owned());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
