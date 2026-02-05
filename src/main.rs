use dioxus::prelude::*;

pub mod audio;
pub mod api;
pub mod models;
pub mod state;
pub mod vad;
pub mod error;
pub mod recording;

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
