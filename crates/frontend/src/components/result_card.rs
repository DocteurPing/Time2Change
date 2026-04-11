//! Recommendation result card component.

use leptos::prelude::*;

use crate::models::{PairAnalysisResponse, RecommendationDto};

/// Renders the recommendation result card when analysis data is available.
///
/// This component is purely presentational and keeps behavior identical to the
/// previous inline implementation in `App`.
#[allow(unreachable_pub)]
#[component]
pub(crate) fn ResultCard(analysis: ReadSignal<Option<PairAnalysisResponse>>) -> impl IntoView {
    let recommendation_text = move || {
        analysis
            .get()
            .map_or("", |result| match result.should_change_now {
                RecommendationDto::ChangeNow => "You should change now.",
                RecommendationDto::Neutral => "You can change now or wait",
                RecommendationDto::Wait => "You should wait.",
            })
    };

    let reasoning_text = move || {
        analysis
            .get()
            .map(|result| result.reasoning)
            .unwrap_or_default()
    };

    let badge_class = move || {
        analysis
            .get()
            .map_or("badge", |result| match result.should_change_now {
                RecommendationDto::ChangeNow => "badge badge-now",
                RecommendationDto::Neutral => "badge badge-neutral",
                RecommendationDto::Wait => "badge badge-later",
            })
    };

    let badge_text = move || {
        analysis
            .get()
            .map_or("", |result| match result.should_change_now {
                RecommendationDto::ChangeNow => "Act now",
                RecommendationDto::Neutral => "Neutral",
                RecommendationDto::Wait => "Wait",
            })
    };

    view! {
        <Show when=move || analysis.get().is_some() fallback=|| ()>
            <article class="result-card">
                <div class="result-header">
                    <h2 class="result-title">"Recommendation"</h2>
                    <span class=badge_class>{badge_text}</span>
                </div>
                <p>{recommendation_text}</p>
                <p class="result-reasoning">{reasoning_text}</p>
            </article>
        </Show>
    }
}
