#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use domain::types::rate_quality_config::RateQualityConfig;
    use rust_decimal::dec;

    use crate::{
        ports::exchange_rate_repository::RepositoryError,
        tests::{
            helpers::{build_rates, make_pair, make_rate},
            mocks::mock_repository::MockRepository,
        },
        use_cases::analyze_pair::{AnalyzeError, AnalyzePairUseCase},
    };

    #[tokio::test]
    async fn execute_returns_analysis_with_correct_pair() {
        let rates = build_rates(&[dec!(1.05), dec!(1.06), dec!(1.07)], 30);
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());
        let pair = make_pair();

        let result = uc.execute(pair.clone(), 30).await.unwrap();

        assert_eq!(result.pair(), &pair);
    }

    #[tokio::test]
    async fn execute_returns_correct_rate_count() {
        let rates = build_rates(&[dec!(1.05), dec!(1.06), dec!(1.07), dec!(1.08)], 30);
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 30).await.unwrap();

        assert_eq!(result.rate_count(), 4);
    }

    #[tokio::test]
    async fn execute_quality_score_is_positive() {
        let rates = build_rates(
            &[dec!(1.05), dec!(1.06), dec!(1.07), dec!(1.08), dec!(1.09)],
            30,
        );
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 30).await.unwrap();

        assert!(*result.quality_score() > dec!(0));
    }

    // ── Tests: should_change_now decision logic ─────────────────────

    #[tokio::test]
    async fn execute_rate_near_top_recommends_change() {
        // Current rate is at the very top of the range → position ≈ 1.0 → should change
        let rates = build_rates(
            &[
                dec!(1.00),
                dec!(1.01),
                dec!(1.02),
                dec!(1.03),
                dec!(1.04),
                dec!(1.05),
                dec!(1.06),
                dec!(1.07),
                dec!(1.08),
                dec!(1.10), // last = current, highest
            ],
            30,
        );
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 30).await.unwrap();

        assert!(result.recommendation().should_change_now());
    }

    #[tokio::test]
    async fn execute_rate_near_bottom_recommends_wait() {
        // Current rate is at the bottom of the range → position ≈ 0.0 → should wait
        let rates = build_rates(
            &[
                dec!(1.10),
                dec!(1.09),
                dec!(1.08),
                dec!(1.07),
                dec!(1.06),
                dec!(1.05),
                dec!(1.04),
                dec!(1.03),
                dec!(1.02),
                dec!(1.00), // last = current, lowest
            ],
            30,
        );
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 30).await.unwrap();

        assert!(!result.recommendation().should_change_now());
    }

    #[tokio::test]
    async fn execute_rate_in_middle_recommends_wait() {
        // Current rate is in the middle → position ≈ 0.5 → should wait (< 0.85)
        let rates = build_rates(
            &[
                dec!(1.00),
                dec!(1.02),
                dec!(1.04),
                dec!(1.06),
                dec!(1.08),
                dec!(1.10),
                dec!(1.08),
                dec!(1.06),
                dec!(1.04),
                dec!(1.05), // middle-ish
            ],
            30,
        );
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 30).await.unwrap();

        assert!(!result.recommendation().should_change_now());
    }

    // ── Tests: reasoning text ───────────────────────────────────────

    #[tokio::test]
    async fn execute_favorable_reasoning_when_should_change() {
        let rates = build_rates(
            &[
                dec!(1.00),
                dec!(1.01),
                dec!(1.02),
                dec!(1.03),
                dec!(1.04),
                dec!(1.05),
                dec!(1.06),
                dec!(1.07),
                dec!(1.08),
                dec!(1.10),
            ],
            30,
        );
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 30).await.unwrap();

        assert!(result.recommendation().reasoning().contains("Favorable"));
    }

    #[tokio::test]
    async fn execute_waiting_reasoning_when_should_not_change() {
        let rates = build_rates(
            &[
                dec!(1.10),
                dec!(1.09),
                dec!(1.08),
                dec!(1.07),
                dec!(1.06),
                dec!(1.05),
                dec!(1.04),
                dec!(1.03),
                dec!(1.02),
                dec!(1.00),
            ],
            30,
        );
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 30).await.unwrap();

        assert!(result.recommendation().reasoning().contains("Waiting"));
    }

    #[tokio::test]
    async fn execute_reasoning_contains_lookback_days() {
        let rates = build_rates(&[dec!(1.05), dec!(1.06), dec!(1.07)], 14);
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 14).await.unwrap();

        assert!(result.recommendation().reasoning().contains("14-day"));
    }

    // ── Tests: confidence ───────────────────────────────────────────

    #[tokio::test]
    async fn execute_confidence_is_non_negative() {
        let rates = build_rates(&[dec!(1.05), dec!(1.06), dec!(1.07), dec!(1.08)], 30);
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 30).await.unwrap();

        assert!(*result.recommendation().confidence() >= dec!(0));
    }

    #[tokio::test]
    async fn execute_strong_signal_has_higher_confidence_than_weak() {
        // Strong signal: rate at top
        let strong_rates = build_rates(
            &[
                dec!(1.00),
                dec!(1.01),
                dec!(1.02),
                dec!(1.03),
                dec!(1.04),
                dec!(1.05),
                dec!(1.06),
                dec!(1.07),
                dec!(1.08),
                dec!(1.10),
            ],
            30,
        );
        let repo_strong = MockRepository::with_rates(strong_rates);
        let uc_strong = AnalyzePairUseCase::new(repo_strong, RateQualityConfig::default());
        let result_strong = uc_strong.execute(make_pair(), 30).await.unwrap();

        // Weak signal: rate exactly in the middle
        let weak_rates = build_rates(
            &[
                dec!(1.00),
                dec!(1.02),
                dec!(1.04),
                dec!(1.06),
                dec!(1.08),
                dec!(1.10),
                dec!(1.08),
                dec!(1.06),
                dec!(1.04),
                dec!(1.05), // mid
            ],
            30,
        );
        let repo_weak = MockRepository::with_rates(weak_rates);
        let uc_weak = AnalyzePairUseCase::new(repo_weak, RateQualityConfig::default());
        let result_weak = uc_weak.execute(make_pair(), 30).await.unwrap();

        assert!(
            result_strong.recommendation().confidence() > result_weak.recommendation().confidence()
        );
    }

    // ── Tests: error paths ──────────────────────────────────────────

    #[tokio::test]
    async fn execute_empty_rates_returns_insufficient_data() {
        let repo = MockRepository::empty();
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let err = uc.execute(make_pair(), 30).await.unwrap_err();

        assert!(matches!(err, AnalyzeError::InsufficientData));
        assert_eq!(err.to_string(), "insufficient historical data");
    }

    #[tokio::test]
    async fn execute_repository_not_found_returns_repository_error() {
        let repo = MockRepository::with_error(RepositoryError::NotFound("EUR-USD".into()));
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let err = uc.execute(make_pair(), 30).await.unwrap_err();

        assert!(matches!(err, AnalyzeError::Repository(_)));
        assert!(err.to_string().contains("EUR-USD"));
    }

    #[tokio::test]
    async fn execute_repository_storage_error_returns_repository_error() {
        let repo =
            MockRepository::with_error(RepositoryError::Storage("connection refused".into()));
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let err = uc.execute(make_pair(), 30).await.unwrap_err();

        assert!(matches!(err, AnalyzeError::Repository(_)));
        assert!(err.to_string().contains("connection refused"));
    }

    #[tokio::test]
    async fn execute_repository_invalid_error_returns_repository_error() {
        let repo = MockRepository::with_error(RepositoryError::Invalid("bad range".into()));
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let err = uc.execute(make_pair(), 30).await.unwrap_err();

        assert!(matches!(err, AnalyzeError::Repository(_)));
        assert!(err.to_string().contains("bad range"));
    }

    // ── Tests: edge cases ───────────────────────────────────────────

    #[tokio::test]
    async fn execute_single_rate_returns_insufficient_data() {
        // A single rate means min == max, so range_position returns None → InsufficientData
        let now = Utc::now();
        let rates = vec![make_rate(now, dec!(1.05))];
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let err = uc.execute(make_pair(), 30).await.unwrap_err();

        assert!(matches!(err, AnalyzeError::InsufficientData));
    }

    #[tokio::test]
    async fn execute_two_identical_rates_does_not_panic() {
        let now = Utc::now();
        let rates = vec![
            make_rate(now - Duration::hours(1), dec!(1.05)),
            make_rate(now, dec!(1.05)),
        ];
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        // When high == low, range_position returns None → InsufficientData
        let err = uc.execute(make_pair(), 30).await.unwrap_err();
        assert!(matches!(err, AnalyzeError::InsufficientData));
    }

    #[tokio::test]
    async fn execute_different_lookback_days() {
        let rates = build_rates(&[dec!(1.05), dec!(1.06), dec!(1.07), dec!(1.08)], 7);
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 7).await.unwrap();

        assert!(result.recommendation().reasoning().contains("7-day"));
    }

    #[tokio::test]
    async fn execute_large_lookback_days() {
        let rates = build_rates(
            &[dec!(1.00), dec!(1.05), dec!(1.10), dec!(1.15), dec!(1.20)],
            365,
        );
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 365).await.unwrap();

        assert!(result.recommendation().reasoning().contains("365-day"));
        assert_eq!(result.rate_count(), 5);
    }

    // ── Tests: error Display ────────────────────────────────────────

    #[tokio::test]
    async fn analyze_error_display_repository() {
        let err = AnalyzeError::Repository(RepositoryError::Storage("broken".into()));
        assert_eq!(err.to_string(), "repository error: storage failure: broken");
    }

    #[tokio::test]
    async fn analyze_error_display_insufficient_data() {
        let err = AnalyzeError::InsufficientData;
        assert_eq!(err.to_string(), "insufficient historical data");
    }

    #[tokio::test]
    async fn analyze_error_debug_impl() {
        let err = AnalyzeError::InsufficientData;
        let debug = format!("{:?}", err);
        assert!(debug.contains("InsufficientData"));
    }

    // ── Tests: recommendation timestamp ─────────────────────────────

    #[tokio::test]
    async fn execute_recommendation_has_recent_timestamp() {
        let before = Utc::now();
        let rates = build_rates(&[dec!(1.05), dec!(1.06), dec!(1.07)], 30);
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 30).await.unwrap();
        let after = Utc::now();

        assert!(*result.recommendation().timestamp() >= before);
        assert!(*result.recommendation().timestamp() <= after);
    }

    // ── Tests: position threshold boundary ──────────────────────────

    #[tokio::test]
    async fn execute_position_exactly_at_085_recommends_change() {
        // Build rates so position is exactly 0.85
        // low=1.00, high=2.00, current needs to be 1.85 → position = (1.85-1.00)/(2.00-1.00) = 0.85
        let now = Utc::now();
        let rates = vec![
            make_rate(now - Duration::days(10), dec!(1.00)), // low
            make_rate(now - Duration::days(9), dec!(2.00)),  // high
            make_rate(now - Duration::days(8), dec!(1.50)),
            make_rate(now - Duration::days(7), dec!(1.60)),
            make_rate(now - Duration::days(6), dec!(1.70)),
            make_rate(now - Duration::days(5), dec!(1.75)),
            make_rate(now - Duration::days(4), dec!(1.80)),
            make_rate(now - Duration::days(3), dec!(1.82)),
            make_rate(now - Duration::days(2), dec!(1.83)),
            make_rate(now - Duration::days(1), dec!(1.84)),
            make_rate(now, dec!(1.85)), // position = 0.85
        ];
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 30).await.unwrap();

        assert!(result.recommendation().should_change_now());
    }

    #[tokio::test]
    async fn execute_position_just_below_085_recommends_wait() {
        // low=1.00, high=2.00, current=1.84 → position = 0.84 < 0.85
        let now = Utc::now();
        let rates = vec![
            make_rate(now - Duration::days(10), dec!(1.00)),
            make_rate(now - Duration::days(9), dec!(2.00)),
            make_rate(now - Duration::days(8), dec!(1.50)),
            make_rate(now - Duration::days(7), dec!(1.60)),
            make_rate(now - Duration::days(6), dec!(1.70)),
            make_rate(now - Duration::days(5), dec!(1.72)),
            make_rate(now - Duration::days(4), dec!(1.74)),
            make_rate(now - Duration::days(3), dec!(1.76)),
            make_rate(now - Duration::days(2), dec!(1.78)),
            make_rate(now - Duration::days(1), dec!(1.80)),
            make_rate(now, dec!(1.84)), // position = 0.84
        ];
        let repo = MockRepository::with_rates(rates);
        let uc = AnalyzePairUseCase::new(repo, RateQualityConfig::default());

        let result = uc.execute(make_pair(), 30).await.unwrap();

        assert!(!result.recommendation().should_change_now());
    }
}
