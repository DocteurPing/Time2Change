//! Main frontend application component.

use leptos::prelude::*;

use crate::components::form::AnalysisForm;
use crate::components::result_card::ResultCard;
use crate::components::status_banner::StatusBanner;
use crate::state::AppState;

/// Root application component.
///
/// Composes the high-level page layout and delegates stateful behaviors to
/// `AppState` and presentational concerns to dedicated child components.
#[allow(unreachable_pub)]
#[component]
pub(crate) fn App() -> impl IntoView {
    let state = AppState::new();

    {
        let state_for_effect = state.clone();
        Effect::new(move |_| {
            state_for_effect.clone().load_currencies();
        });
    }

    let on_analyze = {
        let state = state.clone();
        Callback::new(move |()| {
            state.clone().analyze();
        })
    };

    view! {
        <main class="app-shell">
            <section class="app-container">
                <header class="app-header">
                    <h1 class="app-title">"Time2Change"</h1>
                    <p class="app-subtitle">
                        "Compare two currencies and get a clear recommendation based on recent trends."
                    </p>
                </header>

                <section class="app-main">
                    <AnalysisForm
                        currencies=state.currencies
                        base=state.base
                        quote=state.quote
                        days=state.days
                        status=state.status
                        on_analyze=on_analyze
                    />

                    <StatusBanner status=state.status.read_only() />

                    <ResultCard analysis=state.analysis.read_only() />

                    <p class="footer-note">
                        "This insight supports decision-making and should be combined with your own judgment."
                    </p>
                </section>
            </section>
        </main>
    }
}
