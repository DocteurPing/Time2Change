use application::responses::analyze_pair_responses::Recommendation;

use crate::dto::{HealthCheckResponse, PairAnalysisResponse};

/// Every recommendation must cross the wire as its documented `snake_case` tag.
#[test]
fn recommendation_serializes_as_snake_case() {
    let cases = [
        (Recommendation::ChangeNow, "change_now"),
        (Recommendation::Neutral, "neutral"),
        (Recommendation::Wait, "wait"),
    ];

    for (recommendation, expected) in cases {
        let json = serde_json::to_value(PairAnalysisResponse::new(
            recommendation,
            "because".to_owned(),
        ))
        .unwrap();

        assert_eq!(json["recommendation"], expected);
        assert_eq!(json["reasoning"], "because");
    }
}

#[test]
fn pair_analysis_response_has_no_extra_fields() {
    let json = serde_json::to_value(PairAnalysisResponse::new(
        Recommendation::Wait,
        String::new(),
    ))
    .unwrap();

    let object = json.as_object().unwrap();
    assert_eq!(object.len(), 2);
    assert!(object.contains_key("recommendation"));
    assert!(object.contains_key("reasoning"));
}

#[test]
fn health_check_response_serializes() {
    let json = serde_json::to_value(HealthCheckResponse { is_healthy: true }).unwrap();

    assert_eq!(json["is_healthy"], true);
}
