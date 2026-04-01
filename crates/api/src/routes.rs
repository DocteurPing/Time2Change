use application::ports::currency_repository::CurrencyRepository;
use axum::Json;
use axum::extract::State;

use crate::errors::ApiError;
use crate::state::AppState;

pub(crate) async fn list_currencies(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, ApiError> {
    let currencies = state
        .currency_repo
        .list_currencies()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to fetch currencies: {e}")))?;

    Ok(Json(
        currencies
            .into_iter()
            .map(|currency| currency.code().to_string())
            .collect(),
    ))
}
