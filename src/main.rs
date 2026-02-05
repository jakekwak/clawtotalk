use dioxus::prelude::*;

mod audio;
mod api;
mod models;
mod state;
mod vad;
mod error;

use state::AppState;

fn main() {
    env_logger::init();
    log::info!("Starting Dioxus Voice Assistant");
    
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Initialize global state
    use_context_provider(|| AppState::new());
    
    rsx! {
        div {
            class: "app-container",
            h1 { "Dioxus Voice Assistant" }
            p { "Application is initializing..." }
        }
    }
}
