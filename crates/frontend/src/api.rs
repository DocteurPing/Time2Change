//! Frontend API module for backend HTTP requests.

use std::fmt;

use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::config::API_BASE_URL;
use crate::models::PairAnalysisResponse;

#[derive(Debug, Clone)]
pub(crate) enum ApiError {
    Network(String),
    HttpStatus {
        endpoint: &'static str,
        status: u16,
        body: String,
    },
    Parse {
        endpoint: &'static str,
        message: String,
    },
    Validation(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "Network error: {msg}"),
            Self::HttpStatus {
                endpoint,
                status,
                body,
            } => {
                if body.trim().is_empty() {
                    write!(f, "Request to `{endpoint}` failed with HTTP {status}.")
                } else {
                    write!(
                        f,
                        "Request to `{endpoint}` failed with HTTP {status}: {body}"
                    )
                }
            }
            Self::Parse { endpoint, message } => {
                write!(f, "Could not parse response from `{endpoint}`: {message}")
            }
            Self::Validation(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ApiError {}

type ApiResult<T> = Result<T, ApiError>;

pub(crate) async fn fetch_currencies(client: &Client) -> ApiResult<Vec<String>> {
    let endpoint = "/currencies";
    let url = format!("{API_BASE_URL}{endpoint}");

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(ApiError::HttpStatus {
            endpoint,
            status,
            body,
        });
    }

    let mut currencies: Vec<String> = parse_json(response, endpoint).await?;
    currencies.sort_unstable();
    currencies.dedup();

    if currencies.len() < 2 {
        return Err(ApiError::Validation(
            "Backend returned fewer than 2 currencies.".to_owned(),
        ));
    }

    Ok(currencies)
}

pub(crate) async fn analyze_pair(
    client: &Client,
    base: &str,
    quote: &str,
    days: u32,
) -> ApiResult<PairAnalysisResponse> {
    let endpoint = "/analyze";
    let url = format!("{API_BASE_URL}{endpoint}?base={base}&quote={quote}&days={days}");

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(ApiError::HttpStatus {
            endpoint,
            status,
            body,
        });
    }

    parse_json(response, endpoint).await
}

async fn parse_json<T>(response: reqwest::Response, endpoint: &'static str) -> ApiResult<T>
where
    T: DeserializeOwned,
{
    response.json::<T>().await.map_err(|e| ApiError::Parse {
        endpoint,
        message: e.to_string(),
    })
}
