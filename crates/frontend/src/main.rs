//! frontend

mod api;
mod components;
mod config;
mod models;
mod state;
mod validation;

use components::app::App;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
