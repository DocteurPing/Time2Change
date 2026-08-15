use reqwest::Client;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::api::{ApiError, analyze_pair_from, fetch_currencies_from};
use crate::models::RecommendationDto;

async fn stub() -> (MockServer, Client) {
    let server = MockServer::start().await;
    (server, Client::new())
}

fn json(body: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(body.to_owned(), "application/json")
}

// ── fetch_currencies ────────────────────────────────────────────

#[tokio::test]
async fn fetch_currencies_returns_a_sorted_deduplicated_list() {
    let (server, client) = stub().await;
    Mock::given(method("GET"))
        .and(path("/currencies"))
        .respond_with(json(r#"["USD - US Dollar","EUR - Euro","EUR - Euro"]"#))
        .mount(&server)
        .await;

    let currencies = fetch_currencies_from(&client, &server.uri()).await.unwrap();

    assert_eq!(currencies, vec!["EUR - Euro", "USD - US Dollar"]);
}

#[tokio::test]
async fn fetch_currencies_rejects_a_list_that_cannot_form_a_pair() {
    let (server, client) = stub().await;
    Mock::given(method("GET"))
        .and(path("/currencies"))
        .respond_with(json(r#"["EUR - Euro"]"#))
        .mount(&server)
        .await;

    // The UI needs at least a base and a quote to preselect.
    let error = fetch_currencies_from(&client, &server.uri())
        .await
        .unwrap_err();

    assert!(matches!(error, ApiError::Validation(_)));
    assert_eq!(
        error.to_string(),
        "Backend returned fewer than 2 currencies."
    );
}

#[tokio::test]
async fn fetch_currencies_rejects_duplicates_that_collapse_below_two() {
    let (server, client) = stub().await;
    Mock::given(method("GET"))
        .and(path("/currencies"))
        .respond_with(json(r#"["EUR - Euro","EUR - Euro","EUR - Euro"]"#))
        .mount(&server)
        .await;

    let error = fetch_currencies_from(&client, &server.uri())
        .await
        .unwrap_err();

    assert!(matches!(error, ApiError::Validation(_)));
}

#[tokio::test]
async fn fetch_currencies_surfaces_the_status_and_body_on_failure() {
    let (server, client) = stub().await;
    Mock::given(method("GET"))
        .and(path("/currencies"))
        .respond_with(ResponseTemplate::new(500).set_body_raw("boom", "text/plain"))
        .mount(&server)
        .await;

    let error = fetch_currencies_from(&client, &server.uri())
        .await
        .unwrap_err();

    assert!(
        matches!(
            &error,
            ApiError::HttpStatus { endpoint, status, body }
                if *endpoint == "/currencies" && *status == 500 && body == "boom"
        ),
        "unexpected error: {error:?}"
    );
    assert!(error.to_string().contains("HTTP 500"));
    assert!(error.to_string().contains("boom"));
}

#[tokio::test]
async fn fetch_currencies_omits_an_empty_body_from_the_message() {
    let (server, client) = stub().await;
    Mock::given(method("GET"))
        .and(path("/currencies"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    let error = fetch_currencies_from(&client, &server.uri())
        .await
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "Request to `/currencies` failed with HTTP 503."
    );
}

#[tokio::test]
async fn fetch_currencies_reports_a_parse_failure() {
    let (server, client) = stub().await;
    Mock::given(method("GET"))
        .and(path("/currencies"))
        .respond_with(json(r#"{"not":"a list"}"#))
        .mount(&server)
        .await;

    let error = fetch_currencies_from(&client, &server.uri())
        .await
        .unwrap_err();

    assert!(matches!(error, ApiError::Parse { .. }));
    assert!(error.to_string().contains("Could not parse response"));
}

#[tokio::test]
async fn fetch_currencies_reports_a_network_failure() {
    let client = Client::new();

    // Nothing is listening on this port.
    let error = fetch_currencies_from(&client, "http://127.0.0.1:1")
        .await
        .unwrap_err();

    assert!(matches!(error, ApiError::Network(_)));
    assert!(error.to_string().starts_with("Network error:"));
}

#[tokio::test]
async fn fetch_currencies_reports_a_malformed_base_url() {
    let client = Client::new();

    let error = fetch_currencies_from(&client, "not a url")
        .await
        .unwrap_err();

    assert!(matches!(error, ApiError::Network(_)));
}

// ── analyze_pair ────────────────────────────────────────────────

#[tokio::test]
async fn analyze_pair_sends_the_query_and_parses_the_result() {
    let (server, client) = stub().await;
    Mock::given(method("GET"))
        .and(path("/analyze"))
        .and(query_param("base", "EUR"))
        .and(query_param("quote", "USD"))
        .and(query_param("days", "30"))
        .respond_with(json(
            r#"{"recommendation":"change_now","reasoning":"near the top"}"#,
        ))
        .mount(&server)
        .await;

    let analysis = analyze_pair_from(&client, &server.uri(), "EUR", "USD", 30)
        .await
        .unwrap();

    assert_eq!(analysis.recommendation, RecommendationDto::ChangeNow);
    assert_eq!(analysis.reasoning, "near the top");
}

#[tokio::test]
async fn analyze_pair_parses_every_recommendation_variant() {
    let cases = [
        ("change_now", RecommendationDto::ChangeNow),
        ("neutral", RecommendationDto::Neutral),
        ("wait", RecommendationDto::Wait),
    ];

    for (tag, expected) in cases {
        let (server, client) = stub().await;
        Mock::given(method("GET"))
            .and(path("/analyze"))
            .respond_with(json(&format!(
                r#"{{"recommendation":"{tag}","reasoning":"r"}}"#
            )))
            .mount(&server)
            .await;

        let analysis = analyze_pair_from(&client, &server.uri(), "EUR", "USD", 30)
            .await
            .unwrap();

        assert_eq!(analysis.recommendation, expected, "tag was {tag}");
    }
}

#[tokio::test]
async fn analyze_pair_surfaces_the_backend_error_body() {
    let (server, client) = stub().await;
    Mock::given(method("GET"))
        .and(path("/analyze"))
        .respond_with(ResponseTemplate::new(422).set_body_raw(
            r#"{"error":"Not enough data to analyze pair for the requested range."}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let error = analyze_pair_from(&client, &server.uri(), "EUR", "USD", 30)
        .await
        .unwrap_err();

    assert!(
        matches!(
            &error,
            ApiError::HttpStatus { endpoint, status, .. }
                if *endpoint == "/analyze" && *status == 422
        ),
        "unexpected error: {error:?}"
    );
    assert!(error.to_string().contains("Not enough data"));
}

#[tokio::test]
async fn analyze_pair_reports_a_parse_failure() {
    let (server, client) = stub().await;
    Mock::given(method("GET"))
        .and(path("/analyze"))
        .respond_with(json(r#"{"recommendation":"sideways","reasoning":"r"}"#))
        .mount(&server)
        .await;

    let error = analyze_pair_from(&client, &server.uri(), "EUR", "USD", 30)
        .await
        .unwrap_err();

    assert!(matches!(error, ApiError::Parse { .. }));
}

#[tokio::test]
async fn analyze_pair_reports_a_network_failure() {
    let client = Client::new();

    let error = analyze_pair_from(&client, "http://127.0.0.1:1", "EUR", "USD", 30)
        .await
        .unwrap_err();

    assert!(matches!(error, ApiError::Network(_)));
}

// ── ApiError ────────────────────────────────────────────────────

#[test]
fn validation_errors_render_their_message_verbatim() {
    let error = ApiError::Validation("pick two currencies".to_owned());

    assert_eq!(error.to_string(), "pick two currencies");
}

#[test]
fn api_errors_are_debuggable() {
    let error = ApiError::Network("dns".to_owned());

    assert!(format!("{error:?}").contains("Network"));
}
