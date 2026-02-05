pub mod audio;
pub mod api;
pub mod models;
pub mod state;
pub mod vad;
pub mod error;
pub mod recording;
pub mod ui;
pub mod platform;
pub mod performance;
pub mod memory;
pub mod compression;
pub mod error_handler;
pub mod connection;

use ui::App;

fn main() {
    // Initialize logger with minimal overhead
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    log::info!("Starting Dioxus Voice Assistant");
    
    // Initialize performance monitor
    let _perf = performance::get_performance_monitor();
    
    // Launch app - startup time will be measured in App component
    dioxus::launch(App);
}
