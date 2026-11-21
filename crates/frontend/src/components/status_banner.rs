//! Error status banner component.

#![allow(clippy::empty_enums)]

use leptos::prelude::*;

use crate::models::UiStatus;

/// Renders an error banner when the current UI status is `UiStatus::Error`.
#[allow(unreachable_pub)]
#[component]
pub(crate) fn StatusBanner(status: ReadSignal<UiStatus>) -> impl IntoView {
    view! {
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
    }
}
