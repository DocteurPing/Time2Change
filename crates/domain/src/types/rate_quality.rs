use rust_decimal::Decimal;

/// Detailed component scores that contribute to the overall rate-quality score.
///
/// Each field is expressed as a value in the `0..=100` range, where higher
/// values represent better quality for that dimension.
#[derive(Debug)]
pub struct RateQualityBreakdown {
    completeness: Decimal,    // 0..100
    gap_consistency: Decimal, // 0..100
    outlier: Decimal,         // 0..100
    volatility: Decimal,      // 0..100
}

impl Default for RateQualityBreakdown {
    fn default() -> Self {
        Self {
            completeness: Decimal::ZERO,
            gap_consistency: Decimal::ZERO,
            outlier: Decimal::ZERO,
            volatility: Decimal::ZERO,
        }
    }
}

impl RateQualityBreakdown {
    /// Creates a new quality breakdown from its four component scores.
    ///
    /// The provided values are expected to already be normalized to the
    /// `0..=100` range.
    #[must_use]
    pub const fn new(
        completeness: Decimal,
        gap_consistency: Decimal,
        outlier: Decimal,
        volatility: Decimal,
    ) -> Self {
        Self {
            completeness,
            gap_consistency,
            outlier,
            volatility,
        }
    }

    /// Returns the completeness score.
    #[must_use]
    pub const fn completeness(&self) -> &Decimal {
        &self.completeness
    }

    /// Returns the gap-consistency score.
    #[must_use]
    pub const fn gap_consistency(&self) -> &Decimal {
        &self.gap_consistency
    }

    /// Returns the outlier-resistance score.
    #[must_use]
    pub const fn outlier(&self) -> &Decimal {
        &self.outlier
    }

    /// Returns the volatility score.
    #[must_use]
    pub const fn volatility(&self) -> &Decimal {
        &self.volatility
    }
}

/// Overall rate-quality evaluation for a time series.
///
/// The `overall` score is typically derived from a weighted combination of the
/// values stored in [`RateQualityBreakdown`].
#[derive(Debug)]
pub struct RateQuality {
    overall: Decimal, // 0..100
    breakdown: RateQualityBreakdown,
}

impl Default for RateQuality {
    fn default() -> Self {
        Self {
            overall: Decimal::ZERO,
            breakdown: RateQualityBreakdown::default(),
        }
    }
}

impl RateQuality {
    /// Creates a new overall quality result together with its component
    /// breakdown.
    #[must_use]
    pub const fn new(overall: Decimal, breakdown: RateQualityBreakdown) -> Self {
        Self { overall, breakdown }
    }

    /// Returns the overall quality score.
    #[must_use]
    pub const fn overall(&self) -> &Decimal {
        &self.overall
    }

    /// Returns the component breakdown used to explain the overall score.
    #[must_use]
    pub const fn breakdown(&self) -> &RateQualityBreakdown {
        &self.breakdown
    }
}

#[test]
fn test_rate_quality_default() {
    let rq = RateQuality::default();
    assert_eq!(rq.overall, Decimal::ZERO);
    assert_eq!(rq.breakdown.completeness, Decimal::ZERO);
    assert_eq!(rq.breakdown.gap_consistency, Decimal::ZERO);
    assert_eq!(rq.breakdown.outlier, Decimal::ZERO);
    assert_eq!(rq.breakdown.volatility, Decimal::ZERO);
}
