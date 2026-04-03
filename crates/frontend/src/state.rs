//! Application state and async actions for the frontend.

use leptos::prelude::*;
use reqwest::Client;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::config::DEFAULT_DAYS;
use crate::models::{PairAnalysisResponse, UiStatus};
use crate::validation::validate_analysis_input;

/// Centralized reactive state for the `App`.
#[derive(Clone)]
pub(crate) struct AppState {
    pub currencies: RwSignal<Vec<String>>,
    pub base: RwSignal<String>,
    pub quote: RwSignal<String>,
    pub days: RwSignal<String>,
    pub status: RwSignal<UiStatus>,
    pub analysis: RwSignal<Option<PairAnalysisResponse>>,
}

impl AppState {
    /// Creates a new app state with sensible defaults.
    pub(crate) fn new() -> Self {
        Self {
            currencies: RwSignal::new(Vec::<String>::new()),
            base: RwSignal::new(String::new()),
            quote: RwSignal::new(String::new()),
            days: RwSignal::new(DEFAULT_DAYS.to_string()),
            status: RwSignal::new(UiStatus::Idle),
            analysis: RwSignal::new(None::<PairAnalysisResponse>),
        }
    }

    /// Loads currencies from the backend and initializes default pair selection.
    pub(crate) fn load_currencies(self) {
        self.status.set(UiStatus::LoadingCurrencies);

        let state = self;
        spawn_local(async move {
            let client = Client::new();

            match api::fetch_currencies(&client).await {
                Ok(list) => {
                    let first = list[0].clone();
                    let second = list[1].clone();

                    state.currencies.set(list);
                    state.base.set(first);
                    state.quote.set(second);
                    state.status.set(UiStatus::Ready);
                }
                Err(error) => {
                    state.status.set(UiStatus::Error(error.to_string()));
                }
            }
        });
    }

    /// Validates current form input and calls analyze endpoint.
    pub(crate) fn analyze(self) {
        self.analysis.set(None);

        let current_base = self.base.get_untracked();
        let current_quote = self.quote.get_untracked();
        let current_days_raw = self.days.get_untracked();

        let parsed_days =
            match validate_analysis_input(&current_base, &current_quote, &current_days_raw) {
                Ok(value) => value,
                Err(message) => {
                    self.status.set(UiStatus::Error(message));
                    return;
                }
            };

        self.status.set(UiStatus::Analyzing);

        let state = self;
        spawn_local(async move {
            let client = Client::new();

            match api::analyze_pair(&client, &current_base, &current_quote, parsed_days).await {
                Ok(result) => {
                    state.analysis.set(Some(result));
                    state.status.set(UiStatus::Ready);
                }
                Err(error) => {
                    state.status.set(UiStatus::Error(error.to_string()));
                }
            }
        });
    }
}
