use rust_decimal::{Decimal, dec};
use thiserror::Error;

/// Errors produced while validating rate-quality configuration values.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum RateQualityError {
    /// The configured weights do not add up to `1.0`.
    ///
    /// The rate-quality score is a weighted combination of completeness,
    /// gap consistency, outlier handling, and volatility. To keep the final
    /// score normalized, the sum of all weights must equal exactly `1.0`.
    #[error(
        "Invalid weights: completeness={completeness}, gap_consistency={gap_consistency}, outlier={outlier}, volatility={volatility}"
    )]
    InvalidWeights {
        /// Weight applied to the completeness component.
        completeness: Decimal,
        /// Weight applied to the gap-consistency component.
        gap_consistency: Decimal,
        /// Weight applied to the outlier component.
        outlier: Decimal,
        /// Weight applied to the volatility component.
        volatility: Decimal,
    },

    /// One or more thresholds were non-positive.
    ///
    /// Threshold values must be greater than zero so the scoring rules can
    /// meaningfully distinguish acceptable and unacceptable time-series behavior.
    #[error(
        "Invalid thresholds: outlier_z_threshold={outlier_z_threshold}, max_allowed_volatility={max_allowed_volatility}"
    )]
    InvalidThresholds {
        /// Z-score cutoff used to classify a value as an outlier.
        outlier_z_threshold: Decimal,
        /// Maximum acceptable volatility used by the quality calculation.
        max_allowed_volatility: Decimal,
    },
}

/// Configuration used to evaluate the quality of an exchange-rate time series.
///
/// The configuration is split into:
/// - [`RateQualityWeights`], which determine how much each component contributes
///   to the final score
/// - [`RateQualityThresholds`], which control the sensitivity of outlier and
///   volatility checks
///
/// A default configuration is provided for typical use.
#[derive(Default, Debug)]
pub struct RateQualityConfig {
    weights: RateQualityWeights,
    thresholds: RateQualityThresholds,
}

/// Relative weights for each component of the rate-quality score.
///
/// These weights are expected to sum to exactly `1.0`.
#[derive(Debug)]
pub struct RateQualityWeights {
    completeness: Decimal,
    gap_consistency: Decimal,
    outlier: Decimal,
    volatility: Decimal,
}

/// Threshold values used by rate-quality scoring rules.
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
    /// Creates a new threshold configuration.
    ///
    /// `outlier_z` is the Z-score boundary above which a point is treated as an
    /// outlier. `max_volatility` is the volatility scaling threshold used when
    /// scoring return stability.
    ///
    /// # Errors
    ///
    /// Returns [`RateQualityError::InvalidThresholds`] if either argument is
    /// less than or equal to zero.
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

    /// Returns the Z-score threshold used to classify outliers.
    #[must_use]
    pub const fn outlier_z_threshold(&self) -> Decimal {
        self.outlier_z
    }

    /// Returns the maximum allowed volatility used by the quality model.
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
    /// Creates a new weight configuration.
    ///
    /// The four component weights must sum to exactly `1.0`.
    ///
    /// # Errors
    ///
    /// Returns [`RateQualityError::InvalidWeights`] when the sum of the
    /// provided weights is not equal to `Decimal::ONE`.
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

    /// Returns the completeness weight.
    #[must_use]
    pub const fn completeness(&self) -> Decimal {
        self.completeness
    }

    /// Returns the gap-consistency weight.
    #[must_use]
    pub const fn gap_consistency(&self) -> Decimal {
        self.gap_consistency
    }

    /// Returns the outlier weight.
    #[must_use]
    pub const fn outlier(&self) -> Decimal {
        self.outlier
    }

    /// Returns the volatility weight.
    #[must_use]
    pub const fn volatility(&self) -> Decimal {
        self.volatility
    }
}

impl RateQualityConfig {
    /// Creates a new rate-quality configuration from weights and thresholds.
    #[must_use]
    pub fn new(weights: RateQualityWeights, thresholds: RateQualityThresholds) -> Self {
        Self {
            weights,
            thresholds,
        }
    }

    /// Returns the component weights used by the quality model.
    #[must_use]
    pub const fn weights(&self) -> &RateQualityWeights {
        &self.weights
    }

    /// Returns the thresholds used by the quality model.
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
