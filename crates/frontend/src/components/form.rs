//! Form section component for currency pair analysis.

use leptos::ev;
use leptos::prelude::*;

use crate::config::{MAX_DAYS, MIN_DAYS};
use crate::models::UiStatus;

#[allow(clippy::too_many_lines)]
#[allow(unreachable_pub)]
#[component]
pub(crate) fn AnalysisForm(
    currencies: RwSignal<Vec<String>>,
    base: RwSignal<String>,
    quote: RwSignal<String>,
    days: RwSignal<String>,
    status: RwSignal<UiStatus>,
    on_analyze: Callback<()>,
) -> impl IntoView {
    let is_loading = move || status.get().is_loading();

    view! {
        <div class="form-grid">
            <div class="form-field">
                <label class="field-label" for="base_currency">
                    "Base currency"
                </label>
                <select
                    id="base_currency"
                    class="control"
                    on:change=move |ev: ev::Event| {
                        let target: web_sys::HtmlSelectElement = event_target(&ev);
                        base.set(target.value());
                    }
                    prop:value=move || base.get()
                    disabled=is_loading
                >
                    <For
                        each=move || currencies.get()
                        key=|code| code.clone()
                        children=move |code| {
                            let label = code.clone();
                            view! { <option value=code>{label}</option> }
                        }
                    />
                </select>
            </div>

            <div class="form-field">
                <label class="field-label" for="quote_currency">
                    "Quote currency"
                </label>
                <select
                    id="quote_currency"
                    class="control"
                    on:change=move |ev: ev::Event| {
                        let target: web_sys::HtmlSelectElement = event_target(&ev);
                        quote.set(target.value());
                    }
                    prop:value=move || quote.get()
                    disabled=is_loading
                >
                    <For
                        each=move || currencies.get()
                        key=|code| code.clone()
                        children=move |code| {
                            let label = code.clone();
                            view! { <option value=code>{label}</option> }
                        }
                    />
                </select>
            </div>

            <div class="form-field form-row-full">
                <label class="field-label" for="days">
                    "Lookback days"
                </label>
                <input
                    id="days"
                    class="control"
                    type="number"
                    min=MIN_DAYS
                    max=MAX_DAYS
                    step="1"
                    prop:value=move || days.get()
                    on:input=move |ev: ev::Event| {
                        let target: web_sys::HtmlInputElement = event_target(&ev);
                        days.set(target.value());
                    }
                    disabled=is_loading
                />
                <p class="field-hint">
                    {format!("Enter a value from {MIN_DAYS} to {MAX_DAYS}.")}
                </p>
            </div>
        </div>

        <button
            class="btn-primary"
            on:click=move |_| on_analyze.run(())
            disabled=is_loading
        >
            {move || status.get().cta_label()}
        </button>
    }
}
