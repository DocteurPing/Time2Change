use std::sync::Arc;

use application::ports::currency_repository::CurrencyRepository;
use application::ports::exchange_rate_repository::ExchangeRateRepository;
use application::use_cases::analyze_pair::AnalyzePairUseCase;

#[derive(Clone)]
pub(crate) struct AppState<R, C>
where
    R: ExchangeRateRepository,
    C: CurrencyRepository,
{
    currency_repo: Arc<C>,
    analyzer: Arc<AnalyzePairUseCase<R>>,
}

impl<R, C> AppState<R, C>
where
    R: ExchangeRateRepository,
    C: CurrencyRepository,
{
    pub(crate) const fn new(currency_repo: Arc<C>, analyzer: Arc<AnalyzePairUseCase<R>>) -> Self {
        Self {
            currency_repo,
            analyzer,
        }
    }

    #[must_use]
    pub(crate) fn currency_repo(&self) -> Arc<C> {
        Arc::<C>::clone(&self.currency_repo)
    }

    #[must_use]
    pub(crate) fn analyzer(&self) -> Arc<AnalyzePairUseCase<R>> {
        Arc::<AnalyzePairUseCase<R>>::clone(&self.analyzer)
    }
}
