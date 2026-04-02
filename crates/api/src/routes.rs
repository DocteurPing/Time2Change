use application::ports::currency_repository::CurrencyRepository;
use application::ports::exchange_rate_repository::ExchangeRateRepository;
use application::use_cases::analyze_pair::AnalyzeError;
use axum::Json;
use axum::extract::{Query, State};
use domain::types::currency::Currency;
use domain::types::currency_pair::CurrencyPair;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::errors::ApiError;
use crate::state::AppState;

const MAX_DAYS_ANALYZE: u32 = 365;

pub(crate) async fn list_currencies<R: ExchangeRateRepository>(
    State(state): State<AppState<R>>,
) -> Result<Json<Vec<String>>, ApiError> {
    let currencies = state.currency_repo().list_currencies().await.map_err(|e| {
        error!(error = %e, "Failed to fetch currencies");
        ApiError::Internal("Failed to fetch currencies!".to_owned())
    })?;

    Ok(Json(
        currencies
            .into_iter()
            .map(|currency| currency.code().to_string())
            .collect(),
    ))
}

#[derive(Deserialize)]
pub(crate) struct AnalyzePairQuery {
    base: String,
    quote: String,
    days: u32,
}

#[derive(Serialize)]
pub(crate) struct PairAnalysisResponse {
    should_change_now: bool,
    reasoning: String,
}

pub(crate) async fn analyze_pair<R: ExchangeRateRepository>(
    State(state): State<AppState<R>>,
    query: Query<AnalyzePairQuery>,
) -> Result<Json<PairAnalysisResponse>, ApiError> {
    let base = Currency::new(&query.base).map_err(|e| ApiError::InvalidCurrency(e.to_string()))?;
    let quote =
        Currency::new(&query.quote).map_err(|e| ApiError::InvalidCurrency(e.to_string()))?;
    let pair =
        CurrencyPair::new(base, quote).map_err(|e| ApiError::InvalidCurrencyPair(e.to_string()))?;
    if query.days == 0 || query.days > MAX_DAYS_ANALYZE {
        return Err(ApiError::InvalidCurrency(format!(
            "`days` must be between 1 and {MAX_DAYS_ANALYZE}.",
        )));
    }
    let result = state
        .analyzer()
        .execute(pair, query.days)
        .await
        .map_err(|e| {
            if matches!(e, AnalyzeError::InsufficientData) {
                error!(error = %e, "Not enough data to analyze pair for requested range");
                ApiError::NotEnoughData(
                    "Not enough data to analyze pair for the requested range.".to_owned(),
                )
            } else {
                error!(error = %e, "Failed to analyze pair");
                ApiError::Internal("Failed to analyze pair!".to_owned())
            }
        })?;

    Ok(Json(PairAnalysisResponse {
        should_change_now: result.recommendation().should_change_now(),
        reasoning: result.recommendation().reasoning().to_owned(),
    }))
}
