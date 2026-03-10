use rust_decimal::{Decimal, dec};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
/// Errors that can occur when creating or validating `RateQualityConfig`
pub enum RateQualityError {
    /// Error indicating that the provided weights for completeness, gap consistency, outlier detection, and volatility do not sum to 1.0 (100%).
    #[error(
        "Invalid weights: completeness={completeness}, gap_consistency={gap_consistency}, outlier={outlier}, volatility={volatility}"
    )]
    InvalidWeights {
        /// The weight for completeness in the overall rate quality score.
        completeness: Decimal,
        /// The weight for gap consistency in the overall rate quality score.
        gap_consistency: Decimal,
        /// The weight for outlier detection in the overall rate quality score.
        outlier: Decimal,
        /// The weight for volatility in the overall rate quality score.
        volatility: Decimal,
    },
    /// Error indicating that the provided thresholds for outlier detection and volatility assessment are invalid (e.g., negative or zero values).
    #[error(
        "Invalid thresholds: outlier_z_threshold={outlier_z_threshold}, max_allowed_volatility={max_allowed_volatility}"
    )]
    InvalidThresholds {
        /// The Z-score threshold for outlier detection.
        outlier_z_threshold: Decimal,
        /// The maximum allowed volatility for the time series.
        max_allowed_volatility: Decimal,
    },
}

/// Configuration for rate quality evaluation, including weights for the different components (completeness, gap consistency, outlier detection, and volatility) and thresholds for outlier detection and volatility assessment.
#[derive(Default, Debug)]
pub struct RateQualityConfig {
    weights: RateQualityWeights,
    thresholds: RateQualityThresholds,
}

/// Weights for the different components of the rate quality evaluation: completeness, gap consistency, outlier detection, and volatility.
/// These weights determine the relative importance of each component in the overall rate quality score calculation.
#[derive(Debug)]
pub struct RateQualityWeights {
    completeness: Decimal,
    gap_consistency: Decimal,
    outlier: Decimal,
    volatility: Decimal,
}

/// Thresholds for outlier detection and volatility assessment in the rate quality evaluation.
/// These thresholds determine how sensitive the evaluation is to outliers and volatility in the exchange rate data.
#[derive(Debug)]
pub struct RateQualityThresholds {
    outlier_z: Decimal,
    max_volatility: Decimal,
}

impl Default for RateQualityThresholds {
    fn default() -> Self {
        Self {
            outlier_z: dec!(3.0),
            max_volatility: dec!(1.0),
        }
    }
}

impl RateQualityThresholds {
    /// Create a new `RateQualityThresholds` with the specified Z-score threshold for outlier detection and maximum allowed volatility.
    /// Both thresholds must be greater than 0.
    pub fn new(outlier_z: Decimal, max_volatility: Decimal) -> Result<Self, RateQualityError> {
        if outlier_z <= Decimal::ZERO || max_volatility <= Decimal::ZERO {
            return Err(RateQualityError::InvalidThresholds {
                outlier_z_threshold: outlier_z,
                max_allowed_volatility: max_volatility,
            });
        }
        Ok(Self {
            outlier_z,
            max_volatility,
        })
    }

    /// Get the Z-score threshold for outlier detection. Data points with a Z-score exceeding this threshold will be considered outliers and negatively impact the overall rate quality score.
    #[must_use]
    pub const fn outlier_z_threshold(&self) -> Decimal {
        self.outlier_z
    }

    /// Get the maximum allowed volatility for the time series. If the calculated volatility exceeds this threshold, it will negatively impact the overall rate quality score.
    #[must_use]
    pub const fn max_allowed_volatility(&self) -> Decimal {
        self.max_volatility
    }
}

impl Default for RateQualityWeights {
    fn default() -> Self {
        Self {
            completeness: dec!(0.25),
            gap_consistency: dec!(0.25),
            outlier: dec!(0.25),
            volatility: dec!(0.25),
        }
    }
}

impl RateQualityWeights {
    /// Create a new `RateQualityWeights` with the specified weights for completeness, gap consistency, outlier, and volatility.
    /// The weights must sum to 1.0 (100%).
    pub fn new(
        completeness: Decimal,
        gap_consistency: Decimal,
        outlier: Decimal,
        volatility: Decimal,
    ) -> Result<Self, RateQualityError> {
        if completeness + gap_consistency + outlier + volatility != Decimal::ONE {
            return Err(RateQualityError::InvalidWeights {
                completeness,
                gap_consistency,
                outlier,
                volatility,
            });
        }
        Ok(Self {
            completeness,
            gap_consistency,
            outlier,
            volatility,
        })
    }

    /// Get the weight for completeness in the overall rate quality score.
    #[must_use]
    pub const fn completeness(&self) -> Decimal {
        self.completeness
    }

    /// Get the weight for gap consistency in the overall rate quality score.
    #[must_use]
    pub const fn gap_consistency(&self) -> Decimal {
        self.gap_consistency
    }

    /// Get the weight for outlier detection in the overall rate quality score.
    #[must_use]
    pub const fn outlier(&self) -> Decimal {
        self.outlier
    }

    /// Get the weight for volatility in the overall rate quality score.
    #[must_use]
    pub const fn volatility(&self) -> Decimal {
        self.volatility
    }
}

impl RateQualityConfig {
    /// Create a new `RateQualityConfig` with the specified weights and thresholds.
    #[must_use]
    pub fn new(weights: RateQualityWeights, thresholds: RateQualityThresholds) -> Self {
        Self {
            weights,
            thresholds,
        }
    }

    /// Get the weights for completeness, gap consistency, outlier, and volatility.
    #[must_use]
    pub const fn weights(&self) -> &RateQualityWeights {
        &self.weights
    }

    /// Get the thresholds for outlier detection and volatility assessment.
    #[must_use]
    pub const fn threshold(&self) -> &RateQualityThresholds {
        &self.thresholds
    }
}

#[test]
fn test_rate_quality_config_default() {
    let config = RateQualityConfig::default();
    assert_eq!(config.weights().completeness(), dec!(0.25));
    assert_eq!(config.weights().gap_consistency(), dec!(0.25));
    assert_eq!(config.weights().outlier(), dec!(0.25));
    assert_eq!(config.weights().volatility(), dec!(0.25));
    assert_eq!(config.threshold().outlier_z_threshold(), dec!(3.0));
    assert_eq!(config.threshold().max_allowed_volatility(), dec!(1.0));
}

#[test]
fn test_rate_quality_config_new_valid() {
    let config = RateQualityConfig::new(
        RateQualityWeights::new(dec!(0.2), dec!(0.3), dec!(0.1), dec!(0.4)).unwrap(),
        RateQualityThresholds::new(dec!(2.5), dec!(0.1)).unwrap(),
    );
    assert_eq!(config.weights().completeness(), dec!(0.2));
    assert_eq!(config.weights().gap_consistency(), dec!(0.3));
    assert_eq!(config.weights().outlier(), dec!(0.1));
    assert_eq!(config.weights().volatility(), dec!(0.4));
    assert_eq!(config.threshold().outlier_z_threshold(), dec!(2.5));
    assert_eq!(config.threshold().max_allowed_volatility(), dec!(0.1));
}

#[test]
fn test_rate_quality_config_new_invalid_weights() {
    let err = RateQualityWeights::new(dec!(0.5), dec!(0.5), dec!(0.1), dec!(0.1))
        .err()
        .unwrap();
    assert_eq!(
        err,
        RateQualityError::InvalidWeights {
            completeness: dec!(0.5),
            gap_consistency: dec!(0.5),
            outlier: dec!(0.1),
            volatility: dec!(0.1),
        }
    );
    assert_eq!(
        err.to_string(),
        "Invalid weights: completeness=0.5, gap_consistency=0.5, outlier=0.1, volatility=0.1"
    );
}

#[test]
fn test_rate_quality_config_new_invalid_thresholds() {
    let err = RateQualityThresholds::new(dec!(-1.0), dec!(0.1))
        .err()
        .unwrap();
    assert_eq!(
        err,
        RateQualityError::InvalidThresholds {
            outlier_z_threshold: dec!(-1.0),
            max_allowed_volatility: dec!(0.1)
        }
    );
    assert_eq!(
        err.to_string(),
        "Invalid thresholds: outlier_z_threshold=-1.0, max_allowed_volatility=0.1"
    );
}
