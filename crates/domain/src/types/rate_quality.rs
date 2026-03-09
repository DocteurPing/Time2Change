use rust_decimal::Decimal;

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

    pub const fn completeness(&self) -> &Decimal {
        &self.completeness
    }

    pub const fn gap_consistency(&self) -> &Decimal {
        &self.gap_consistency
    }

    pub const fn outlier(&self) -> &Decimal {
        &self.outlier
    }

    pub const fn volatility(&self) -> &Decimal {
        &self.volatility
    }
}

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
    pub const fn new(overall: Decimal, breakdown: RateQualityBreakdown) -> Self {
        Self { overall, breakdown }
    }

    pub const fn overall(&self) -> &Decimal {
        &self.overall
    }

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
