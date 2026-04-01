use std::sync::Arc;

use infrastructure::currency::repository::PostgresCurrencyRepository;

#[derive(Clone)]
pub(crate) struct AppState {
    pub currency_repo: Arc<PostgresCurrencyRepository>,
}
