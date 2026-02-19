use rust_decimal::Decimal;

pub struct RateQualityBreakdown {
    pub completeness: Decimal,    // 0..100
    pub gap_consistency: Decimal, // 0..100
    pub outlier: Decimal,         // 0..100
    pub volatility: Decimal,      // 0..100
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

pub struct RateQuality {
    pub overall: Decimal, // 0..100
    pub breakdown: RateQualityBreakdown,
}

impl Default for RateQuality {
    fn default() -> Self {
        Self {
            overall: Decimal::ZERO,
            breakdown: RateQualityBreakdown::default(),
        }
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
