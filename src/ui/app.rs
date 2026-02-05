use dioxus::prelude::*;
use dioxus_router::{Routable, Router, navigator};
use crate::state::AppState;
use crate::models::ConnectionStatus;
use super::{RecordingButton, ConversationHistory, SettingsScreen, ConnectionStatusDisplay, ErrorNotification};

/// Main application routes
#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/settings")]
    Settings {},
}

/// Main application component
/// Requirement 7.1: Provides consistent user experience across all platforms
#[component]
pub fn App() -> Element {
    // Initialize global state
    use_context_provider(|| AppState::new());
    use_context_provider(|| Signal::new(ConnectionStatus::Disconnected));
    use_context_provider(|| Signal::new(Vec::<String>::new())); // Error notifications
    
    rsx! {
        Router::<Route> {}
    }
}

/// Home screen component
#[component]
fn Home() -> Element {
    let nav = navigator();
    let mut app_state = use_context::<AppState>();
    let connection_status = use_context::<Signal<ConnectionStatus>>();
    let mut error_notifications = use_context::<Signal<Vec<String>>>();
    
    // Clone values before using in closures
    let recording_mode = app_state.recording_mode.read().clone();
    let is_recording = *app_state.is_recording.read();
    let messages = app_state.conversation_history.read().clone();
    let conn_status = connection_status.read().clone();
    let errors = error_notifications.read().clone();
    
    rsx! {
        div {
            class: "app-container",
            style: "display: flex; flex-direction: column; height: 100vh; padding: 20px; box-sizing: border-box;",
            
            // Header with connection status and settings button
            div {
                class: "header",
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                
                h1 {
                    style: "margin: 0; font-size: 24px;",
                    "Voice Assistant"
                }
                
                div {
                    style: "display: flex; align-items: center; gap: 15px;",
                    
                    ConnectionStatusDisplay {
                        status: conn_status
                    }
                    
                    button {
                        onclick: move |_| {
                            nav.push(Route::Settings {});
                        },
                        style: "padding: 8px 16px; cursor: pointer; border: 1px solid #ccc; border-radius: 4px; background: white;",
                        "⚙️ Settings"
                    }
                }
            }
            
            // Error notifications
            div {
                class: "error-notifications",
                style: "margin-bottom: 10px;",
                for (idx, error) in errors.iter().enumerate() {
                    ErrorNotification {
                        key: "{idx}",
                        message: error.clone(),
                        on_dismiss: {
                            let error_to_remove = error.clone();
                            move |_| {
                                error_notifications.write().retain(|e| e != &error_to_remove);
                            }
                        }
                    }
                }
            }
            
            // Conversation history (scrollable)
            div {
                class: "conversation-container",
                style: "flex: 1; overflow-y: auto; margin-bottom: 20px; border: 1px solid #e0e0e0; border-radius: 8px; padding: 15px; background: #f9f9f9;",
                
                ConversationHistory {
                    messages: messages
                }
            }
            
            // Recording button (fixed at bottom)
            div {
                class: "recording-controls",
                style: "display: flex; justify-content: center; align-items: center;",
                
                RecordingButton {
                    recording_mode: recording_mode,
                    is_recording: is_recording,
                    on_toggle: move |_| {
                        app_state.toggle_recording();
                    }
                }
            }
        }
    }
}

/// Settings screen component wrapper
#[component]
fn Settings() -> Element {
    let nav = navigator();
    let mut app_state = use_context::<AppState>();
    
    // Clone settings before using in closure
    let settings = app_state.settings.read().clone();
    
    rsx! {
        div {
            class: "settings-container",
            style: "padding: 20px; height: 100vh; box-sizing: border-box; overflow-y: auto;",
            
            div {
                class: "settings-header",
                style: "display: flex; align-items: center; margin-bottom: 20px;",
                
                button {
                    onclick: move |_| {
                        nav.push(Route::Home {});
                    },
                    style: "padding: 8px 16px; cursor: pointer; border: 1px solid #ccc; border-radius: 4px; background: white; margin-right: 15px;",
                    "← Back"
                }
                
                h1 {
                    style: "margin: 0; font-size: 24px;",
                    "Settings"
                }
            }
            
            SettingsScreen {
                settings: settings,
                on_save: move |new_settings| {
                    // Requirement 8.5: Settings changes are applied immediately
                    app_state.update_settings(new_settings);
                    nav.push(Route::Home {});
                }
            }
        }
    }
}
