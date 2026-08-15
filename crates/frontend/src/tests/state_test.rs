use leptos::prelude::*;

use crate::config::DEFAULT_DAYS;
use crate::models::UiStatus;
use crate::state::AppState;

/// Signals need a reactive owner; `Owner::new` supplies one outside the browser.
fn with_owner<T>(f: impl FnOnce() -> T) -> T {
    let owner = Owner::new();
    owner.set();
    let result = f();
    drop(owner);
    result
}

#[test]
fn a_new_state_starts_idle_with_no_selection() {
    with_owner(|| {
        let state = AppState::new();

        assert_eq!(state.status.get_untracked(), UiStatus::Idle);
        assert!(state.currencies.get_untracked().is_empty());
        assert!(state.base.get_untracked().is_empty());
        assert!(state.quote.get_untracked().is_empty());
        assert!(state.analysis.get_untracked().is_none());
    });
}

#[test]
fn a_new_state_prefills_the_default_lookback() {
    with_owner(|| {
        let state = AppState::new();

        assert_eq!(state.days.get_untracked(), DEFAULT_DAYS.to_string());
    });
}
