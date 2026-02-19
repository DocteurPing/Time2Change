use rust_decimal::{Decimal, dec};

#[derive(Debug, PartialEq, Eq)]
pub enum RateQualityError {
    InvalidWeights,
    InvalidThresholds,
}

pub struct RateQualityConfig {
    pub w_completeness: Decimal,
    pub w_gap_consistency: Decimal,
    pub w_outlier: Decimal,
    pub w_volatility: Decimal,
    pub outlier_z_threshold: Decimal,
    pub max_allowed_volatility: Decimal,
}

impl Default for RateQualityConfig {
    fn default() -> Self {
        Self {
            w_completeness: dec!(0.25),
            w_gap_consistency: dec!(0.25),
            w_outlier: dec!(0.25),
            w_volatility: dec!(0.25),
            outlier_z_threshold: dec!(3.0),
            max_allowed_volatility: dec!(1.0),
        }
    }
}

impl RateQualityConfig {
    pub fn new(
        w_completeness: Decimal,
        w_gap_consistency: Decimal,
        w_outlier: Decimal,
        w_volatility: Decimal,
        outlier_z_threshold: Decimal,
        max_allowed_volatility: Decimal,
    ) -> Result<Self, RateQualityError> {
        if w_completeness + w_gap_consistency + w_outlier + w_volatility != Decimal::ONE {
            return Err(RateQualityError::InvalidWeights);
        }
        if outlier_z_threshold <= Decimal::ZERO || max_allowed_volatility <= Decimal::ZERO {
            return Err(RateQualityError::InvalidThresholds);
        }
        Ok(Self {
            w_completeness,
            w_gap_consistency,
            w_outlier,
            w_volatility,
            outlier_z_threshold,
            max_allowed_volatility,
        })
    }
}

#[test]
fn test_rate_quality_config_default() {
    let config = RateQualityConfig::default();
    assert_eq!(config.w_completeness, dec!(0.25));
    assert_eq!(config.w_gap_consistency, dec!(0.25));
    assert_eq!(config.w_outlier, dec!(0.25));
    assert_eq!(config.w_volatility, dec!(0.25));
    assert_eq!(config.outlier_z_threshold, dec!(3.0));
}

#[test]
fn test_rate_quality_config_new_valid() {
    let config = RateQualityConfig::new(
        dec!(0.2),
        dec!(0.3),
        dec!(0.1),
        dec!(0.4),
        dec!(2.5),
        dec!(0.1),
    )
    .unwrap();
    assert_eq!(config.w_completeness, dec!(0.2));
    assert_eq!(config.w_gap_consistency, dec!(0.3));
    assert_eq!(config.w_outlier, dec!(0.1));
    assert_eq!(config.w_volatility, dec!(0.4));
    assert_eq!(config.outlier_z_threshold, dec!(2.5));
    assert_eq!(config.max_allowed_volatility, dec!(0.1));
}

#[test]
fn test_rate_quality_config_new_invalid_weights() {
    let err = RateQualityConfig::new(
        dec!(0.5),
        dec!(0.5),
        dec!(0.1),
        dec!(0.1),
        dec!(2.5),
        dec!(0.1),
    )
    .err()
    .unwrap();
    assert!(matches!(err, RateQualityError::InvalidWeights));
}

#[test]
fn test_rate_quality_config_new_invalid_thresholds() {
    let err = RateQualityConfig::new(
        dec!(0.25),
        dec!(0.25),
        dec!(0.25),
        dec!(0.25),
        dec!(-1.0),
        dec!(0.1),
    )
    .err()
    .unwrap();
    assert_eq!(err, RateQualityError::InvalidThresholds);
}
