use dioxus::prelude::*;
use crate::models::{Message, Settings, AppStatus, RecordingMode};

/// Global application state
#[derive(Clone)]
pub struct AppState {
    pub recording_mode: Signal<RecordingMode>,
    pub is_recording: Signal<bool>,
    pub conversation_history: Signal<Vec<Message>>,
    pub settings: Signal<Settings>,
    pub current_status: Signal<AppStatus>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            recording_mode: Signal::new(RecordingMode::default()),
            is_recording: Signal::new(false),
            conversation_history: Signal::new(Vec::new()),
            settings: Signal::new(Settings::default()),
            current_status: Signal::new(AppStatus::Idle),
        }
    }
    
    pub fn add_message(&mut self, message: Message) {
        self.conversation_history.write().push(message);
    }
    
    pub fn set_status(&mut self, status: AppStatus) {
        *self.current_status.write() = status;
    }
    
    pub fn toggle_recording(&mut self) {
        let current = *self.is_recording.read();
        *self.is_recording.write() = !current;
    }
    
    pub fn update_settings(&mut self, settings: Settings) {
        *self.settings.write() = settings;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
