use dioxus::prelude::*;
use crate::models::{Message, Settings, AppStatus, RecordingMode};
use crate::memory::PaginatedHistory;

/// Global application state
#[derive(Clone)]
pub struct AppState {
    pub recording_mode: Signal<RecordingMode>,
    pub is_recording: Signal<bool>,
    pub conversation_history: Signal<PaginatedHistory<Message>>,
    pub settings: Signal<Settings>,
    pub current_status: Signal<AppStatus>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            recording_mode: Signal::new(RecordingMode::default()),
            is_recording: Signal::new(false),
            // Requirement 11.3: Use paginated history for memory efficiency
            conversation_history: Signal::new(PaginatedHistory::new(50, 500)),
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
    
    pub fn start_recording(&mut self) {
        *self.is_recording.write() = true;
        self.set_status(AppStatus::Recording);
    }
    
    pub fn stop_recording(&mut self) {
        *self.is_recording.write() = false;
        self.set_status(AppStatus::Idle);
    }
    
    pub fn update_settings(&mut self, settings: Settings) {
        // Requirement 8.5: Settings changes are applied immediately
        *self.settings.write() = settings.clone();
        *self.recording_mode.write() = settings.recording_mode;
    }
    
    pub fn clear_conversation(&mut self) {
        self.conversation_history.write().clear();
    }
    
    pub fn get_message_count(&self) -> usize {
        self.conversation_history.read().len()
    }
    
    /// Get recent messages for display (memory efficient)
    pub fn get_recent_messages(&self, count: usize) -> Vec<Message> {
        self.conversation_history.read().get_recent(count)
    }
    
    /// Get a specific page of messages
    pub fn get_message_page(&self, page: usize) -> Vec<Message> {
        self.conversation_history.read().get_page(page)
    }
    
    /// Trim conversation history to a specific count
    pub fn trim_conversation(&mut self, max_count: usize) {
        self.conversation_history.write().trim_to(max_count);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

