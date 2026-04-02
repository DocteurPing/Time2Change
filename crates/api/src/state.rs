use std::sync::Arc;

use application::ports::exchange_rate_repository::ExchangeRateRepository;
use application::use_cases::analyze_pair::AnalyzePairUseCase;
use infrastructure::currency::repository::PostgresCurrencyRepository;

#[derive(Clone)]
pub(crate) struct AppState<R>
where
    R: ExchangeRateRepository,
{
    currency_repo: Arc<PostgresCurrencyRepository>,
    analyzer: Arc<AnalyzePairUseCase<R>>,
}

impl<R> AppState<R>
where
    R: ExchangeRateRepository,
{
    pub(crate) const fn new(
        currency_repo: Arc<PostgresCurrencyRepository>,
        analyzer: Arc<AnalyzePairUseCase<R>>,
    ) -> Self {
        Self {
            currency_repo,
            analyzer,
        }
    }

    #[must_use]
    pub(crate) fn currency_repo(&self) -> Arc<PostgresCurrencyRepository> {
        Arc::<PostgresCurrencyRepository>::clone(&self.currency_repo)
    }

    #[must_use]
    pub(crate) fn analyzer(&self) -> Arc<AnalyzePairUseCase<R>> {
        Arc::<AnalyzePairUseCase<R>>::clone(&self.analyzer)
    }
}
