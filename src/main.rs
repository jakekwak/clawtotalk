use dioxus::prelude::*;

pub mod audio;
pub mod api;
pub mod models;
pub mod state;
pub mod vad;
pub mod error;
pub mod recording;
pub mod ui;
pub mod platform;

use ui::App;

fn main() {
    env_logger::init();
    log::info!("Starting Dioxus Voice Assistant");
    
    dioxus::launch(App);
}
