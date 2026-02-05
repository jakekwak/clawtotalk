// UI module for Dioxus Voice Assistant
pub mod app;
pub mod recording_button;
pub mod conversation_history;
pub mod settings_screen;
pub mod connection_status;
pub mod error_notification;

pub use app::App;
pub use recording_button::RecordingButton;
pub use conversation_history::ConversationHistory;
pub use settings_screen::SettingsScreen;
pub use connection_status::ConnectionStatusDisplay;
pub use error_notification::ErrorNotification;
