use crate::models::{PairAnalysisResponse, RecommendationDto, UiStatus};

// ── UiStatus ────────────────────────────────────────────────────

#[test]
fn only_in_flight_states_count_as_loading() {
    assert!(UiStatus::LoadingCurrencies.is_loading());
    assert!(UiStatus::Analyzing.is_loading());

    assert!(!UiStatus::Idle.is_loading());
    assert!(!UiStatus::Ready.is_loading());
    assert!(!UiStatus::Error("boom".to_owned()).is_loading());
}

#[test]
fn the_cta_label_reflects_the_in_flight_operation() {
    assert_eq!(UiStatus::Analyzing.cta_label(), "Analyzing...");
    assert_eq!(
        UiStatus::LoadingCurrencies.cta_label(),
        "Loading currencies..."
    );
}

#[test]
fn the_cta_label_falls_back_to_the_default_prompt() {
    for status in [
        UiStatus::Idle,
        UiStatus::Ready,
        UiStatus::Error("boom".to_owned()),
    ] {
        assert_eq!(status.cta_label(), "Analyze pair");
    }
}

#[test]
fn an_error_state_carries_its_message() {
    let status = UiStatus::Error("network down".to_owned());

    assert_eq!(status, UiStatus::Error("network down".to_owned()));
    assert_ne!(status, UiStatus::Error("something else".to_owned()));
    assert!(format!("{status:?}").contains("network down"));
}

// ── PairAnalysisResponse ────────────────────────────────────────

#[test]
fn the_response_round_trips_through_json() {
    let original = PairAnalysisResponse {
        recommendation: RecommendationDto::Wait,
        reasoning: "sits low in the range".to_owned(),
    };

    let encoded = serde_json::to_string(&original).unwrap();
    let decoded: PairAnalysisResponse = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded, original);
}

#[test]
fn the_recommendation_tag_matches_the_backend_wire_format() {
    let cases = [
        (RecommendationDto::ChangeNow, "change_now"),
        (RecommendationDto::Neutral, "neutral"),
        (RecommendationDto::Wait, "wait"),
    ];

    for (variant, tag) in cases {
        let response = PairAnalysisResponse {
            recommendation: variant,
            reasoning: String::new(),
        };
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["recommendation"], tag);
    }
}

#[test]
fn an_unknown_recommendation_tag_is_rejected() {
    let result = serde_json::from_str::<PairAnalysisResponse>(
        r#"{"recommendation":"sideways","reasoning":""}"#,
    );

    assert!(result.is_err());
}
