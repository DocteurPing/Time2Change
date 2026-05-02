//! Frontend domain and UI models.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RecommendationDto {
    ChangeNow,
    Neutral,
    Wait,
}

/// Response payload returned by the backend analyze endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PairAnalysisResponse {
    pub recommendation: RecommendationDto,
    pub reasoning: String,
}

/// High-level UI state for async workflows and user feedback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UiStatus {
    Idle,
    LoadingCurrencies,
    Ready,
    Analyzing,
    Error(String),
}

impl UiStatus {
    /// Returns `true` when a background operation is in progress.
    pub(crate) const fn is_loading(&self) -> bool {
        matches!(self, Self::LoadingCurrencies | Self::Analyzing)
    }

    /// Call-to-action button label based on current state.
    pub(crate) const fn cta_label(&self) -> &'static str {
        match self {
            Self::Analyzing => "Analyzing...",
            Self::LoadingCurrencies => "Loading currencies...",
            _ => "Analyze pair",
        }
    }
}
