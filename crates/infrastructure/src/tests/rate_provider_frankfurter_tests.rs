use application::ports::rate_provider::{RateProvider, RateProviderError};
use chrono::TimeZone;
use domain::types::currency_pair::CurrencyPair;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::repositories::rate_provider::frankfurter::FrankfurterClient;

use domain::types::currency::Currency;

async fn mock_server_with(_body: &str) -> (MockServer, FrankfurterClient) {
    let server = MockServer::start().await;
    let client = FrankfurterClient::with_base_url(server.uri()).unwrap();
    (server, client)
}

#[tokio::test]
async fn create_client_test() {
    let client = FrankfurterClient::default();
    assert_eq!(client.base_url(), "https://api.frankfurter.app");
}

#[tokio::test]
async fn fetch_latest_returns_exchange_rate() {
    let (server, client) = mock_server_with("").await;
    Mock::given(method("GET"))
        .and(path("/latest"))
        .and(query_param("base", "EUR"))
        .and(query_param("symbols", "USD"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"base":"EUR","date":"2024-01-15","rates":{"USD":1.0945}}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();

    let rate = client.fetch_latest(&pair).await.unwrap();
    assert_eq!(
        rate.timestamp(),
        &chrono::Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap()
    );
    assert_eq!(rate.rate(), &rust_decimal::Decimal::new(10945, 4));
}

#[tokio::test]
async fn get_rate_returns_historical_exchange_rate() {
    let (server, client) = mock_server_with("").await;
    Mock::given(method("GET"))
        .and(path("/2024-01-01"))
        .and(query_param("base", "EUR"))
        .and(query_param("symbols", "USD"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"base":"EUR","date":"2024-01-01","rates":{"USD":1.0850}}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();

    let rate = client
        .get_rate(
            &pair,
            chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        rate.timestamp(),
        &chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
    );
    assert_eq!(rate.rate(), &rust_decimal::Decimal::new(10850, 4));
}

#[tokio::test]
async fn returns_pair_not_supported_on_404() {
    let (server, client) = mock_server_with("").await;
    Mock::given(method("GET"))
        .and(path("/latest"))
        .and(query_param("base", "EUR"))
        .and(query_param("symbols", "USD"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();
    let error = client.fetch_latest(&pair).await;
    assert!(matches!(error, Err(RateProviderError::PairNotSupported(_))));
}

#[tokio::test]
async fn returns_api_error_on_500() {
    let (server, client) = mock_server_with("").await;
    Mock::given(method("GET"))
        .and(path("/latest"))
        .and(query_param("base", "EUR"))
        .and(query_param("symbols", "USD"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();
    let error = client.fetch_latest(&pair).await;
    assert!(matches!(error, Err(RateProviderError::ApiError(_))));
}

#[tokio::test]
async fn returns_timeout_on_request_timeout() {
    let (server, client) = mock_server_with("").await;
    Mock::given(method("GET"))
        .and(path("/latest"))
        .and(query_param("base", "EUR"))
        .and(query_param("symbols", "USD"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(10)))
        .mount(&server)
        .await;

    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();
    let error = client.fetch_latest(&pair).await;
    assert!(matches!(error, Err(RateProviderError::Timeout)));
}

#[tokio::test]
async fn returns_pair_not_supported_when_quote_missing_from_rates() {
    let (server, client) = mock_server_with("").await;
    Mock::given(method("GET"))
        .and(path("/latest"))
        .and(query_param("base", "EUR"))
        .and(query_param("symbols", "USD"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            // "rates" exists but doesn't have "USD"
            r#"{"base":"EUR","date":"2024-01-15","rates":{"GBP":0.85}}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();
    let error = client.fetch_latest(&pair).await;
    assert!(matches!(error, Err(RateProviderError::PairNotSupported(_))));
}

#[tokio::test]
async fn returns_api_error_on_connection_refused() {
    let client = FrankfurterClient::with_base_url("http://127.0.0.1:1").unwrap();

    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();

    let error = client.fetch_latest(&pair).await;
    assert!(matches!(error, Err(RateProviderError::ApiError(_))));
}

#[tokio::test]
async fn returns_parse_error_on_invalid_json() {
    let (server, client) = mock_server_with("").await;
    Mock::given(method("GET"))
        .and(path("/latest"))
        .and(query_param("base", "EUR"))
        .and(query_param("symbols", "USD"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("{}", "application/json"))
        .mount(&server)
        .await;

    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();
    let error = client.fetch_latest(&pair).await;
    assert!(matches!(error, Err(RateProviderError::ParseError(_))));
}

#[tokio::test]
async fn returns_parse_error_on_non_decimal_rate() {
    let (server, client) = mock_server_with("").await;
    Mock::given(method("GET"))
        .and(path("/latest"))
        .and(query_param("base", "EUR"))
        .and(query_param("symbols", "USD"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"base":"EUR","date":"2024-01-15","rates":{"USD":"not a number"}}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();
    let error = client.fetch_latest(&pair).await;
    assert!(matches!(error, Err(RateProviderError::ParseError(_))));
}
