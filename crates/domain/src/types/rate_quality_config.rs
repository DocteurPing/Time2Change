use rust_decimal::{Decimal, dec};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum RateQualityError {
    #[error(
        "Invalid weights: completeness={completeness}, gap_consistency={gap_consistency}, outlier={outlier}, volatility={volatility}"
    )]
    InvalidWeights {
        completeness: Decimal,
        gap_consistency: Decimal,
        outlier: Decimal,
        volatility: Decimal,
    },
    #[error(
        "Invalid thresholds: outlier_z_threshold={outlier_z_threshold}, max_allowed_volatility={max_allowed_volatility}"
    )]
    InvalidThresholds {
        outlier_z_threshold: Decimal,
        max_allowed_volatility: Decimal,
    },
}

#[derive(Default)]
pub struct RateQualityConfig {
    weights: RateQualityWeights,
    thresholds: RateQualityThresholds,
}

pub struct RateQualityWeights {
    completeness: Decimal,
    gap_consistency: Decimal,
    outlier: Decimal,
    volatility: Decimal,
}

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

    pub const fn outlier_z_threshold(&self) -> Decimal {
        self.outlier_z
    }

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

    pub const fn completeness(&self) -> Decimal {
        self.completeness
    }

    pub const fn gap_consistency(&self) -> Decimal {
        self.gap_consistency
    }

    pub const fn outlier(&self) -> Decimal {
        self.outlier
    }

    pub const fn volatility(&self) -> Decimal {
        self.volatility
    }
}

impl RateQualityConfig {
    pub fn new(weights: RateQualityWeights, thresholds: RateQualityThresholds) -> Self {
        Self {
            weights,
            thresholds,
        }
    }

    pub const fn weights(&self) -> &RateQualityWeights {
        &self.weights
    }

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
