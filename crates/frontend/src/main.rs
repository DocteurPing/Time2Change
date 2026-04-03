//! frontend

use leptos::ev;
use leptos::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

const API_BASE_URL: &str = "http://127.0.0.1:3000";
const DEFAULT_DAYS: u32 = 30;
const MIN_DAYS: u32 = 1;
const MAX_DAYS: u32 = 365;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PairAnalysisResponse {
    should_change_now: bool,
    reasoning: String,
}

#[derive(Debug, Clone)]
enum UiStatus {
    Idle,
    LoadingCurrencies,
    CurrenciesLoaded,
    Analyzing,
    Error(String),
}

impl UiStatus {
    const fn is_loading(&self) -> bool {
        matches!(self, Self::LoadingCurrencies | Self::Analyzing)
    }

    const fn cta_label<'a>(&self) -> &'a str {
        match self {
            Self::Analyzing => "Analyzing...",
            Self::LoadingCurrencies => "Loading currencies...",
            _ => "Analyze pair",
        }
    }
}

#[component]
fn App() -> impl IntoView {
    let currencies = RwSignal::new(Vec::<String>::new());
    let base = RwSignal::new(String::new());
    let quote = RwSignal::new(String::new());
    let days = RwSignal::new(DEFAULT_DAYS.to_string());
    let status = RwSignal::new(UiStatus::Idle);
    let analysis = RwSignal::new(None::<PairAnalysisResponse>);

    let load_currencies = {
        // let currencies = currencies;
        // let base = base;
        // let quote = quote;
        // let status = status;

        move || {
            status.set(UiStatus::LoadingCurrencies);

            spawn_local(async move {
                let client = Client::new();
                let url = format!("{API_BASE_URL}/currencies");

                match client.get(url).send().await {
                    Ok(resp) => {
                        if !resp.status().is_success() {
                            status.set(UiStatus::Error(format!(
                                "Failed to load currencies (HTTP {}).",
                                resp.status()
                            )));
                            return;
                        }

                        match resp.json::<Vec<String>>().await {
                            Ok(mut list) => {
                                list.sort_unstable();
                                list.dedup();

                                if list.len() < 2 {
                                    status.set(UiStatus::Error(
                                        "Backend returned fewer than 2 currencies.".to_owned(),
                                    ));
                                    return;
                                }

                                let first = list[0].clone();
                                let second = list[1].clone();

                                currencies.set(list);
                                base.set(first);
                                quote.set(second);
                                status.set(UiStatus::CurrenciesLoaded);
                            }
                            Err(e) => {
                                status.set(UiStatus::Error(format!(
                                    "Could not parse currencies response: {e}",
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        status.set(UiStatus::Error(format!(
                            "Could not contact API to load currencies: {e}",
                        )));
                    }
                }
            });
        }
    };

    Effect::new(move |_| {
        load_currencies();
    });

    let on_analyze = {
        // let base = base;
        // let quote = quote;
        // let days = days;
        // let analysis = analysis;
        // let status = status;

        move |_| {
            analysis.set(None);

            let current_base = base.get_untracked();
            let current_quote = quote.get_untracked();
            let current_days_raw = days.get_untracked();

            if current_base.is_empty() || current_quote.is_empty() {
                status.set(UiStatus::Error(
                    "Please select both base and quote currencies.".to_owned(),
                ));
                return;
            }

            if current_base == current_quote {
                status.set(UiStatus::Error(
                    "Base and quote currency must be different.".to_owned(),
                ));
                return;
            }

            let parsed_days = match current_days_raw.trim().parse::<u32>() {
                Ok(value) if (MIN_DAYS..=MAX_DAYS).contains(&value) => value,
                _ => {
                    status.set(UiStatus::Error(format!(
                        "`days` must be a number between {MIN_DAYS} and {MAX_DAYS}.",
                    )));
                    return;
                }
            };

            status.set(UiStatus::Analyzing);

            spawn_local(async move {
                let client = Client::new();
                let url = format!(
                    "{API_BASE_URL}/analyze?base={current_base}&quote={current_quote}&days={parsed_days}",
                );

                match client.get(url).send().await {
                    Ok(resp) => {
                        if !resp.status().is_success() {
                            let status_code = resp.status();
                            let body = resp.text().await.unwrap_or_default();
                            let message = if body.trim().is_empty() {
                                format!("Analysis failed (HTTP {status_code}).")
                            } else {
                                format!("Analysis failed (HTTP {status_code}): {body}")
                            };
                            status.set(UiStatus::Error(message));
                            return;
                        }

                        match resp.json::<PairAnalysisResponse>().await {
                            Ok(result) => {
                                analysis.set(Some(result));
                                status.set(UiStatus::CurrenciesLoaded);
                            }
                            Err(e) => {
                                status.set(UiStatus::Error(format!(
                                    "Could not parse analysis response: {e}",
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        status.set(UiStatus::Error(format!(
                            "Could not contact API to analyze currency pair: {e}",
                        )));
                    }
                }
            });
        }
    };

    let recommendation_text = move || {
        analysis.get().map_or("", |result| {
            if result.should_change_now {
                "You should change now."
            } else {
                "You should wait."
            }
        })
    };

    let reasoning_text = move || {
        analysis
            .get()
            .map(|result| result.reasoning)
            .unwrap_or_default()
    };

    let badge_class = move || {
        analysis.get().map_or("badge", |result| {
            if result.should_change_now {
                "badge badge-now"
            } else {
                "badge badge-later"
            }
        })
    };

    let badge_text = move || {
        analysis.get().map_or("", |result| {
            if result.should_change_now {
                "Act now"
            } else {
                "Wait"
            }
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
                                disabled=move || status.get().is_loading()
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
                                disabled=move || status.get().is_loading()
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
                                disabled=move || status.get().is_loading()
                            />
                            <p class="field-hint">
                                "Enter a value from 1 to 365."
                            </p>
                        </div>
                    </div>

                    <button
                        class="btn-primary"
                        on:click=on_analyze
                        disabled=move || status.get().is_loading()
                    >
                        {move || status.get().cta_label()}
                    </button>

                    <Show
                        when=move || matches!(status.get(), UiStatus::Error(_))
                        fallback=|| ()
                    >
                        <div class="notice notice-error" role="alert">
                            <p>
                                {move || match status.get() {
                                    UiStatus::Error(message) => message,
                                    _ => String::new(),
                                }}
                            </p>
                        </div>
                    </Show>

                    <Show
                        when=move || analysis.get().is_some()
                        fallback=|| ()
                    >
                        <article class="result-card">
                            <div class="result-header">
                                <h2 class="result-title">"Recommendation"</h2>
                                <span class=badge_class>{badge_text}</span>
                            </div>
                            <p class="decision">{recommendation_text}</p>
                            <p class="result-reasoning">{reasoning_text}</p>
                        </article>
                    </Show>

                    <p class="footer-note">
                        "This insight supports decision-making and should be combined with your own judgment."
                    </p>
                </section>
            </section>
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();

    #[cfg(target_arch = "wasm32")]
    {
        mount_to_body(App);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("This frontend is a WASM app. Use `trunk serve` to run it in the browser.");
    }
}
