use application::ports::rate_provider::{RateProvider, RateProviderError};
use chrono::{NaiveDate, TimeZone};
use domain::types::currency::Currency;
use domain::types::currency_info::CurrencyInfo;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use rust_decimal::{Decimal, dec};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::rate_provider::frankfurter::FrankfurterClient;

async fn mock_server() -> (MockServer, FrankfurterClient) {
    let server = MockServer::start().await;
    let client = FrankfurterClient::with_base_url_and_timeout(server.uri(), 100).unwrap();
    (server, client)
}

#[tokio::test]
async fn create_client_test() {
    let client = FrankfurterClient::with_default_url().unwrap();
    assert_eq!(client.base_url(), "https://api.frankfurter.dev/v2");
}

#[tokio::test]
async fn fetch_latest_returns_exchange_rate() {
    let (server, client) = mock_server().await;
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
    assert_eq!(rate.rate(), &Decimal::new(10945, 4));
}

#[tokio::test]
async fn get_rate_returns_historical_exchange_rate() {
    let (server, client) = mock_server().await;
    Mock::given(method("GET"))
        .and(path("/1999-01-01"))
        .and(query_param("base", "USD"))
        .and(query_param("symbols", "EUR"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
            "base": "USD",
            "date": "1999-01-01",
            "rates": {
                "EUR": 0.84825
            }
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let pair =
        CurrencyPair::new(Currency::new("USD").unwrap(), Currency::new("EUR").unwrap()).unwrap();

    let rate = client
        .get_rate(
            &pair,
            chrono::Utc.with_ymd_and_hms(1999, 1, 1, 0, 0, 0).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        rate.timestamp(),
        &chrono::Utc.with_ymd_and_hms(1999, 1, 1, 0, 0, 0).unwrap()
    );
    assert_eq!(rate.rate(), &Decimal::new(84825, 5));
}

#[tokio::test]
async fn returns_pair_not_supported_on_404() {
    let (server, client) = mock_server().await;
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
    let (server, client) = mock_server().await;
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
    let (server, client) = mock_server().await;
    Mock::given(method("GET"))
        .and(path("/latest"))
        .and(query_param("base", "EUR"))
        .and(query_param("symbols", "USD"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_millis(200)))
        .mount(&server)
        .await;

    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();
    let error = client.fetch_latest(&pair).await;
    assert!(matches!(error, Err(RateProviderError::Timeout)));
}

#[tokio::test]
async fn returns_pair_not_supported_when_quote_missing_from_rates() {
    let (server, client) = mock_server().await;
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
    let client = FrankfurterClient::with_base_url_and_timeout("http://127.0.0.1:1", 5000).unwrap();

    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();

    let error = client.fetch_latest(&pair).await;
    assert!(matches!(error, Err(RateProviderError::ApiError(_))));
}

#[tokio::test]
async fn returns_parse_error_on_invalid_json() {
    let (server, client) = mock_server().await;
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
    let (server, client) = mock_server().await;
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

#[tokio::test]
async fn fetch_currencies() {
    let (server, client) = mock_server().await;
    Mock::given(method("GET"))
        .and(path("/currencies"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[
              {
                "iso_code": "AED",
                "iso_numeric": "784",
                "name": "United Arab Emirates Dirham",
                "symbol": "د.إ",
                "...": "..."
              },
              {
                "iso_code": "AFN",
                "iso_numeric": "971",
                "name": "Afghan Afghani",
                "symbol": "؋",
                "...": "..."
              },
              {
                "iso_code": "ALL",
                "iso_numeric": "008",
                "name": "Albanian Lek",
                "symbol": "L",
                "...": "..."
              },
              {
                "iso_code": "AMD",
                "iso_numeric": "051",
                "name": "Armenian Dram",
                "symbol": "֏",
                "...": "..."
              }
            ]"#,
            "application/json",
        ))
        .mount(&server)
        .await;
    let currencies = client.fetch_currencies().await.unwrap();
    assert!(currencies.contains(&CurrencyInfo::new(
        Currency::try_from("AED").unwrap(),
        "United Arab Emirates Dirham".to_owned()
    )));
    assert!(currencies.contains(&CurrencyInfo::new(
        Currency::try_from("AMD").unwrap(),
        "Armenian Dram".to_owned()
    )));
}

#[tokio::test]
async fn fetch_currencies_parse_error() {
    let (server, client) = mock_server().await;
    Mock::given(method("GET"))
        .and(path("/currencies"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(r#"{"EUR"}"#, "application/json"))
        .mount(&server)
        .await;
    let error = client.fetch_currencies().await;
    assert!(matches!(error, Err(RateProviderError::ParseError(_))));
}

#[tokio::test]
async fn fetch_currencies_parse_error_currency_format() {
    let (server, client) = mock_server().await;
    Mock::given(method("GET"))
        .and(path("/currencies"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[
          {
            "iso_code": "AEDD",
            "iso_numeric": "784",
            "name": "United Arab Emirates Dirham",
            "symbol": "د.إ",
            "...": "..."
          }]"#,
            "application/json",
        ))
        .mount(&server)
        .await;
    let error = client.fetch_currencies().await;
    assert!(matches!(error, Err(RateProviderError::ParseError(_))));
}

#[tokio::test]
async fn fetch_currencies_api_error() {
    let client = FrankfurterClient::with_base_url_and_timeout("http://127.0.0.1:1", 5000).unwrap();
    let error = client.fetch_currencies().await;
    assert!(matches!(error, Err(RateProviderError::ApiError(_))));
}

#[tokio::test]
async fn fetch_currencies_return_api_error_on_500() {
    let (server, client) = mock_server().await;
    Mock::given(method("GET"))
        .and(path("/currencies"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let error = client.fetch_currencies().await;
    assert!(matches!(error, Err(RateProviderError::ApiError(_))));
}

#[tokio::test]
async fn fetch_currencies_return_timeout_on_timeout() {
    let (server, client) = mock_server().await;
    Mock::given(method("GET"))
        .and(path("/currencies"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_millis(200)))
        .mount(&server)
        .await;

    let error = client.fetch_currencies().await;
    assert!(matches!(error, Err(RateProviderError::Timeout)));
}

#[tokio::test]
async fn fetch_currencies_returns_parse_error_on_invalid_currency_code() {
    let (server, client) = mock_server().await;
    Mock::given(method("GET"))
        .and(path("/currencies"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
            "AUDE": "Australian Dollar"
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let error = client.fetch_currencies().await;
    assert!(matches!(error, Err(RateProviderError::ParseError(_))));
}

#[allow(clippy::too_many_lines)]
#[tokio::test]
async fn get_rates_for_range_returns_ok_on_valid_response() {
    let (server, client) = mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rates"))
        .and(query_param("from", "2024-01-01"))
        .and(query_param("to", "2024-01-05"))
        .and(query_param("base", "EUR"))
        .and(query_param("quotes", "USD"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[
              {
                "date": "2024-01-01",
                "base": "EUR",
                "quote": "USD",
                "rate": 1.1077
              },
              {
                "date": "2024-01-02",
                "base": "EUR",
                "quote": "USD",
                "rate": 1.1024
              },
              {
                "date": "2024-01-03",
                "base": "EUR",
                "quote": "USD",
                "rate": 1.0936
              },
              {
                "date": "2024-01-04",
                "base": "EUR",
                "quote": "USD",
                "rate": 1.092
              }
            ]"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();
    let result = client
        .get_rates_for_range(
            &pair,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
        )
        .await;
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.len() == 4);
    assert!(result.contains(&ExchangeRate::new(
        chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        dec!(1.1077)
    )));
    assert!(result.contains(&ExchangeRate::new(
        chrono::Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(),
        dec!(1.1024)
    )));
    assert!(result.contains(&ExchangeRate::new(
        chrono::Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(),
        dec!(1.0936)
    )));
    assert!(result.contains(&ExchangeRate::new(
        chrono::Utc.with_ymd_and_hms(2024, 1, 4, 0, 0, 0).unwrap(),
        dec!(1.092)
    )));
}

#[tokio::test]
async fn get_rates_for_range_returns_err_on_invalid_response() {
    let (server, client) = mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rates"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
            "AUDE": "Australian Dollar"
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;
    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();
    let result = client
        .get_rates_for_range(
            &pair,
            NaiveDate::from_ymd_opt(2023, 12, 29).unwrap(),
            NaiveDate::from_ymd_opt(2026, 3, 24).unwrap(),
        )
        .await;
    assert!(result.is_err_and(|e| matches!(e, RateProviderError::ParseError(_))));
}

#[tokio::test]
async fn get_rates_for_range_returns_err_on_wrong_date() {
    let (server, client) = mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rates"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
                "base": "EUR",
                "start_date": "2023-12-29",
                "end_date": "2026-03-24",
                "rates": {
                  "2023-22-29": {
                    "USD": 1.105
                  }
                }
              }"#,
            "application/json",
        ))
        .mount(&server)
        .await;
    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();
    let result = client
        .get_rates_for_range(
            &pair,
            NaiveDate::from_ymd_opt(2023, 12, 29).unwrap(),
            NaiveDate::from_ymd_opt(2026, 3, 24).unwrap(),
        )
        .await;
    assert!(result.is_err_and(|e| matches!(e, RateProviderError::ParseError(_))));
}

#[tokio::test]
async fn get_rates_for_range_returns_err_on_wrong_pair() {
    let (server, client) = mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rates"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[
              {
                "date": "2024-01-01",
                "base": "EUR",
                "quote": "EUR",
                "rate": 1.1077
              }
            ]"#,
            "application/json",
        ))
        .mount(&server)
        .await;
    let pair =
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap();
    let result = client
        .get_rates_for_range(
            &pair,
            NaiveDate::from_ymd_opt(2023, 12, 29).unwrap(),
            NaiveDate::from_ymd_opt(2026, 3, 24).unwrap(),
        )
        .await;
    assert!(result.is_err_and(|e| matches!(e, RateProviderError::PairNotSupported(_))));
}
