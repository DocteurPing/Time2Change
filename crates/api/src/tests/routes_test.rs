use std::sync::Arc;

use application::use_cases::analyze_pair::AnalyzePairUseCase;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use domain::types::rate_quality_config::RateQualityConfig;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use crate::routes::{analyze_pair, health, list_currencies};
use crate::state::AppState;
use crate::tests::mocks::{MockCurrencyRepository, MockRateRepository};

/// Builds the same router `main` wires up, over the supplied repositories.
fn app(rates: MockRateRepository, currencies: MockCurrencyRepository) -> Router {
    let state = AppState::new(
        Arc::new(currencies),
        Arc::new(AnalyzePairUseCase::new(rates, RateQualityConfig::default())),
    );

    Router::new()
        .route("/currencies", get(list_currencies))
        .route("/analyze", get(analyze_pair))
        .route("/health", get(health))
        .with_state(state)
}

/// A router with data good enough for a successful analysis.
fn healthy_app() -> Router {
    app(
        MockRateRepository::rising(),
        MockCurrencyRepository::with_currencies(&[("EUR", "Euro"), ("USD", "US Dollar")]),
    )
}

async fn get_request(app: Router, uri: &str) -> (StatusCode, Value) {
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);

    (status, body)
}

// ── /health ─────────────────────────────────────────────────────

#[tokio::test]
async fn health_reports_healthy() {
    let (status, body) = get_request(healthy_app(), "/health").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_healthy"], Value::Bool(true));
}

// ── /currencies ─────────────────────────────────────────────────

#[tokio::test]
async fn list_currencies_returns_code_and_name() {
    let (status, body) = get_request(healthy_app(), "/currencies").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!(["EUR - Euro", "USD - US Dollar"]));
}

#[tokio::test]
async fn list_currencies_returns_an_empty_list_when_none_are_stored() {
    let app = app(
        MockRateRepository::empty(),
        MockCurrencyRepository::with_currencies(&[]),
    );
    let (status, body) = get_request(app, "/currencies").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!([]));
}

#[tokio::test]
async fn list_currencies_maps_repository_failures_to_500() {
    let app = app(
        MockRateRepository::empty(),
        MockCurrencyRepository::failing(),
    );
    let (status, body) = get_request(app, "/currencies").await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    // The client is told something failed, never what the storage layer said.
    assert_eq!(body["error"], "Failed to fetch currencies!");
    assert!(!body["error"].as_str().unwrap().contains("connection lost"));
}

// ── /analyze — success ──────────────────────────────────────────

#[tokio::test]
async fn analyze_recommends_changing_when_the_rate_is_near_its_high() {
    let (status, body) = get_request(healthy_app(), "/analyze?base=EUR&quote=USD&days=30").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["recommendation"], "change_now");
    assert!(!body["reasoning"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn analyze_recommends_waiting_when_the_rate_is_near_its_low() {
    let app = app(
        MockRateRepository::falling(),
        MockCurrencyRepository::with_currencies(&[("EUR", "Euro")]),
    );
    let (status, body) = get_request(app, "/analyze?base=EUR&quote=USD&days=30").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["recommendation"], "wait");
}

#[tokio::test]
async fn analyze_accepts_the_boundary_lookback_values() {
    for days in [1, 365] {
        let (status, _) = get_request(
            healthy_app(),
            &format!("/analyze?base=EUR&quote=USD&days={days}"),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "days={days} should be accepted");
    }
}

// ── /analyze — validation ───────────────────────────────────────

#[tokio::test]
async fn analyze_rejects_a_malformed_currency() {
    let (status, body) = get_request(healthy_app(), "/analyze?base=EURO&quote=USD&days=30").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("3 letters"));
}

#[tokio::test]
async fn analyze_rejects_a_lowercase_currency() {
    let (status, _) = get_request(healthy_app(), "/analyze?base=eur&quote=USD&days=30").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn analyze_rejects_a_malformed_quote_currency() {
    let (status, _) = get_request(healthy_app(), "/analyze?base=EUR&quote=U&days=30").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn analyze_rejects_identical_currencies() {
    let (status, body) = get_request(healthy_app(), "/analyze?base=EUR&quote=EUR&days=30").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("cannot be the same")
    );
}

#[tokio::test]
async fn analyze_rejects_out_of_range_lookbacks() {
    for days in [0, 366, 100_000] {
        let (status, body) = get_request(
            healthy_app(),
            &format!("/analyze?base=EUR&quote=USD&days={days}"),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "days={days} should be rejected"
        );
        assert!(
            body["error"]
                .as_str()
                .unwrap()
                .contains("between 1 and 365")
        );
    }
}

#[tokio::test]
async fn analyze_rejects_missing_query_parameters() {
    for uri in [
        "/analyze",
        "/analyze?base=EUR",
        "/analyze?base=EUR&quote=USD",
        "/analyze?quote=USD&days=30",
    ] {
        let (status, _) = get_request(healthy_app(), uri).await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri} should be rejected");
    }
}

#[tokio::test]
async fn analyze_rejects_a_non_numeric_lookback() {
    let (status, _) = get_request(healthy_app(), "/analyze?base=EUR&quote=USD&days=soon").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ── /analyze — failures ─────────────────────────────────────────

#[tokio::test]
async fn analyze_reports_422_when_there_is_no_data() {
    let app = app(
        MockRateRepository::empty(),
        MockCurrencyRepository::with_currencies(&[("EUR", "Euro")]),
    );
    let (status, body) = get_request(app, "/analyze?base=EUR&quote=USD&days=30").await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body["error"].as_str().unwrap().contains("Not enough data"));
}

#[tokio::test]
async fn analyze_reports_422_when_the_rate_never_moves() {
    // A flat series has no min/max range, so position cannot be derived.
    let app = app(
        MockRateRepository::with_values(&[
            rust_decimal::dec!(1.1),
            rust_decimal::dec!(1.1),
            rust_decimal::dec!(1.1),
        ]),
        MockCurrencyRepository::with_currencies(&[("EUR", "Euro")]),
    );
    let (status, _) = get_request(app, "/analyze?base=EUR&quote=USD&days=30").await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn analyze_maps_repository_failures_to_500() {
    let app = app(
        MockRateRepository::failing(),
        MockCurrencyRepository::with_currencies(&[("EUR", "Euro")]),
    );
    let (status, body) = get_request(app, "/analyze?base=EUR&quote=USD&days=30").await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"], "Failed to analyze pair!");
    assert!(!body["error"].as_str().unwrap().contains("connection lost"));
}

// ── Routing ─────────────────────────────────────────────────────

#[tokio::test]
async fn unknown_paths_return_404() {
    let (status, _) = get_request(healthy_app(), "/nope").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
