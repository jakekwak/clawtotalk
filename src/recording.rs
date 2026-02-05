use crate::audio::AudioManager;
use crate::error::AudioError;
use crate::models::RecordingMode;
use crate::vad::{VoiceActivityDetector, VadResult};
use std::sync::Arc;

/// Recording mode handler trait
#[async_trait::async_trait]
pub trait RecordingModeHandler: Send + Sync {
    /// Handle button press event
    async fn on_button_press(&mut self) -> Result<(), AudioError>;
    
    /// Handle button release event
    async fn on_button_release(&mut self) -> Result<Option<Vec<u8>>, AudioError>;
    
    /// Process audio frame (for Auto mode)
    async fn process_audio_frame(&mut self, frame: &[f32]) -> Result<(), AudioError>;
    
    /// Check if currently recording
    fn is_recording(&self) -> bool;
    
    /// Get current audio level
    fn get_audio_level(&self) -> f32;
    
    /// Reset the handler state
    fn reset(&mut self);
}

/// Hold mode handler
/// 
/// In Hold mode, recording starts when the button is pressed and stops when released.
/// This provides immediate feedback and control to the user.
pub struct HoldModeHandler<T: AudioManager> {
    audio_manager: Arc<T>,
    is_recording: bool,
}

impl<T: AudioManager> HoldModeHandler<T> {
    pub fn new(audio_manager: Arc<T>) -> Self {
        Self {
            audio_manager,
            is_recording: false,
        }
    }
}

#[async_trait::async_trait]
impl<T: AudioManager> RecordingModeHandler for HoldModeHandler<T> {
    async fn on_button_press(&mut self) -> Result<(), AudioError> {
        if !self.is_recording {
            log::info!("Hold mode: Starting recording on button press");
            self.audio_manager.start_recording().await?;
            self.is_recording = true;
        }
        Ok(())
    }
    
    async fn on_button_release(&mut self) -> Result<Option<Vec<u8>>, AudioError> {
        if self.is_recording {
            log::info!("Hold mode: Stopping recording on button release");
            let audio_data = self.audio_manager.stop_recording().await?;
            self.is_recording = false;
            Ok(Some(audio_data))
        } else {
            Ok(None)
        }
    }
    
    async fn process_audio_frame(&mut self, _frame: &[f32]) -> Result<(), AudioError> {
        // Hold mode doesn't process audio frames
        Ok(())
    }
    
    fn is_recording(&self) -> bool {
        self.is_recording
    }
    
    fn get_audio_level(&self) -> f32 {
        self.audio_manager.get_audio_level()
    }
    
    fn reset(&mut self) {
        self.is_recording = false;
    }
}

/// Toggle mode handler
/// 
/// In Toggle mode, the first click starts recording and the second click stops it.
/// This is useful for hands-free operation after the initial click.
pub struct ToggleModeHandler<T: AudioManager> {
    audio_manager: Arc<T>,
    is_recording: bool,
}

impl<T: AudioManager> ToggleModeHandler<T> {
    pub fn new(audio_manager: Arc<T>) -> Self {
        Self {
            audio_manager,
            is_recording: false,
        }
    }
}

#[async_trait::async_trait]
impl<T: AudioManager> RecordingModeHandler for ToggleModeHandler<T> {
    async fn on_button_press(&mut self) -> Result<(), AudioError> {
        // Toggle mode handles everything on press, release is ignored
        Ok(())
    }
    
    async fn on_button_release(&mut self) -> Result<Option<Vec<u8>>, AudioError> {
        if self.is_recording {
            // Stop recording
            log::info!("Toggle mode: Stopping recording");
            let audio_data = self.audio_manager.stop_recording().await?;
            self.is_recording = false;
            Ok(Some(audio_data))
        } else {
            // Start recording
            log::info!("Toggle mode: Starting recording");
            self.audio_manager.start_recording().await?;
            self.is_recording = true;
            Ok(None)
        }
    }
    
    async fn process_audio_frame(&mut self, _frame: &[f32]) -> Result<(), AudioError> {
        // Toggle mode doesn't process audio frames
        Ok(())
    }
    
    fn is_recording(&self) -> bool {
        self.is_recording
    }
    
    fn get_audio_level(&self) -> f32 {
        self.audio_manager.get_audio_level()
    }
    
    fn reset(&mut self) {
        self.is_recording = false;
    }
}

/// Auto mode handler
/// 
/// In Auto mode, recording starts automatically when speech is detected
/// and stops after a period of silence. This provides a fully hands-free experience.
pub struct AutoModeHandler<T: AudioManager> {
    audio_manager: Arc<T>,
    vad: VoiceActivityDetector,
    is_recording: bool,
    audio_buffer: Vec<u8>,
}

impl<T: AudioManager> AutoModeHandler<T> {
    pub fn new(audio_manager: Arc<T>, vad: VoiceActivityDetector) -> Self {
        Self {
            audio_manager,
            vad,
            is_recording: false,
            audio_buffer: Vec::new(),
        }
    }
}

#[async_trait::async_trait]
impl<T: AudioManager> RecordingModeHandler for AutoModeHandler<T> {
    async fn on_button_press(&mut self) -> Result<(), AudioError> {
        // Auto mode doesn't use button events
        Ok(())
    }
    
    async fn on_button_release(&mut self) -> Result<Option<Vec<u8>>, AudioError> {
        // Auto mode doesn't use button events
        Ok(None)
    }
    
    async fn process_audio_frame(&mut self, frame: &[f32]) -> Result<(), AudioError> {
        let vad_result = self.vad.analyze_frame(frame);
        
        match vad_result {
            VadResult::Speech => {
                if !self.is_recording {
                    log::info!("Auto mode: Speech detected, starting recording");
                    self.audio_manager.start_recording().await?;
                    self.is_recording = true;
                }
            }
            VadResult::Silence => {
                if self.is_recording && !self.vad.is_speech_detected() {
                    log::info!("Auto mode: Silence detected, stopping recording");
                    let audio_data = self.audio_manager.stop_recording().await?;
                    self.is_recording = false;
                    self.audio_buffer = audio_data;
                }
            }
            VadResult::Unknown => {
                // Do nothing for unknown results
            }
        }
        
        Ok(())
    }
    
    fn is_recording(&self) -> bool {
        self.is_recording
    }
    
    fn get_audio_level(&self) -> f32 {
        self.audio_manager.get_audio_level()
    }
    
    fn reset(&mut self) {
        self.is_recording = false;
        self.audio_buffer.clear();
        self.vad.reset();
    }
}

/// Recording mode manager
/// 
/// Manages the current recording mode and delegates events to the appropriate handler.
pub struct RecordingModeManager<T: AudioManager> {
    current_mode: RecordingMode,
    hold_handler: HoldModeHandler<T>,
    toggle_handler: ToggleModeHandler<T>,
    auto_handler: AutoModeHandler<T>,
}

impl<T: AudioManager> RecordingModeManager<T> {
    pub fn new(
        audio_manager: Arc<T>,
        vad: VoiceActivityDetector,
    ) -> Self {
        Self {
            current_mode: RecordingMode::Hold,
            hold_handler: HoldModeHandler::new(audio_manager.clone()),
            toggle_handler: ToggleModeHandler::new(audio_manager.clone()),
            auto_handler: AutoModeHandler::new(audio_manager, vad),
        }
    }
    
    pub fn set_mode(&mut self, mode: RecordingMode) {
        log::info!("Switching recording mode to: {:?}", mode);
        
        // Reset all handlers when switching modes
        self.hold_handler.reset();
        self.toggle_handler.reset();
        self.auto_handler.reset();
        
        self.current_mode = mode;
    }
    
    pub fn get_mode(&self) -> &RecordingMode {
        &self.current_mode
    }
    
    pub async fn on_button_press(&mut self) -> Result<(), AudioError> {
        match self.current_mode {
            RecordingMode::Hold => self.hold_handler.on_button_press().await,
            RecordingMode::Toggle => self.toggle_handler.on_button_press().await,
            RecordingMode::Auto => self.auto_handler.on_button_press().await,
        }
    }
    
    pub async fn on_button_release(&mut self) -> Result<Option<Vec<u8>>, AudioError> {
        match self.current_mode {
            RecordingMode::Hold => self.hold_handler.on_button_release().await,
            RecordingMode::Toggle => self.toggle_handler.on_button_release().await,
            RecordingMode::Auto => self.auto_handler.on_button_release().await,
        }
    }
    
    pub async fn process_audio_frame(&mut self, frame: &[f32]) -> Result<(), AudioError> {
        match self.current_mode {
            RecordingMode::Hold => self.hold_handler.process_audio_frame(frame).await,
            RecordingMode::Toggle => self.toggle_handler.process_audio_frame(frame).await,
            RecordingMode::Auto => self.auto_handler.process_audio_frame(frame).await,
        }
    }
    
    pub fn is_recording(&self) -> bool {
        match self.current_mode {
            RecordingMode::Hold => self.hold_handler.is_recording(),
            RecordingMode::Toggle => self.toggle_handler.is_recording(),
            RecordingMode::Auto => self.auto_handler.is_recording(),
        }
    }
    
    pub fn get_audio_level(&self) -> f32 {
        match self.current_mode {
            RecordingMode::Hold => self.hold_handler.get_audio_level(),
            RecordingMode::Toggle => self.toggle_handler.get_audio_level(),
            RecordingMode::Auto => self.auto_handler.get_audio_level(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::CrossPlatformAudioManager;
    use crate::models::VadSettings;
    
    #[tokio::test]
    async fn test_hold_mode_button_press_starts_recording() {
        let audio_manager = Arc::new(CrossPlatformAudioManager::new().unwrap());
        let mut handler = HoldModeHandler::new(audio_manager);
        
        assert!(!handler.is_recording());
        
        handler.on_button_press().await.unwrap();
        assert!(handler.is_recording());
    }
    
    #[tokio::test]
    async fn test_hold_mode_button_release_stops_recording() {
        let audio_manager = Arc::new(CrossPlatformAudioManager::new().unwrap());
        let mut handler = HoldModeHandler::new(audio_manager);
        
        // Start recording
        handler.on_button_press().await.unwrap();
        assert!(handler.is_recording());
        
        // Stop recording
        let result = handler.on_button_release().await.unwrap();
        assert!(!handler.is_recording());
        assert!(result.is_some());
    }
    
    #[tokio::test]
    async fn test_hold_mode_release_without_press() {
        let audio_manager = Arc::new(CrossPlatformAudioManager::new().unwrap());
        let mut handler = HoldModeHandler::new(audio_manager);
        
        let result = handler.on_button_release().await.unwrap();
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_toggle_mode_first_click_starts_recording() {
        let audio_manager = Arc::new(CrossPlatformAudioManager::new().unwrap());
        let mut handler = ToggleModeHandler::new(audio_manager);
        
        assert!(!handler.is_recording());
        
        let result = handler.on_button_release().await.unwrap();
        assert!(handler.is_recording());
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_toggle_mode_second_click_stops_recording() {
        let audio_manager = Arc::new(CrossPlatformAudioManager::new().unwrap());
        let mut handler = ToggleModeHandler::new(audio_manager);
        
        // First click - start
        handler.on_button_release().await.unwrap();
        assert!(handler.is_recording());
        
        // Second click - stop
        let result = handler.on_button_release().await.unwrap();
        assert!(!handler.is_recording());
        assert!(result.is_some());
    }
    
    #[tokio::test]
    async fn test_auto_mode_speech_detection_starts_recording() {
        let audio_manager = Arc::new(CrossPlatformAudioManager::new().unwrap());
        let vad = VoiceActivityDetector::new(VadSettings::default());
        let mut handler = AutoModeHandler::new(audio_manager, vad);
        
        assert!(!handler.is_recording());
        
        // Establish baseline with low energy
        for _ in 0..5 {
            let low_frame = vec![0.001; 1024];
            handler.process_audio_frame(&low_frame).await.unwrap();
        }
        
        // Feed speech frame (high energy)
        let speech_frame = vec![0.5; 1024];
        handler.process_audio_frame(&speech_frame).await.unwrap();
        
        assert!(handler.is_recording());
    }
    
    #[tokio::test]
    async fn test_recording_mode_manager_mode_switching() {
        let audio_manager = Arc::new(CrossPlatformAudioManager::new().unwrap());
        let vad = VoiceActivityDetector::new(VadSettings::default());
        let mut manager = RecordingModeManager::new(audio_manager, vad);
        
        assert_eq!(*manager.get_mode(), RecordingMode::Hold);
        
        manager.set_mode(RecordingMode::Toggle);
        assert_eq!(*manager.get_mode(), RecordingMode::Toggle);
        
        manager.set_mode(RecordingMode::Auto);
        assert_eq!(*manager.get_mode(), RecordingMode::Auto);
    }
    
    #[tokio::test]
    async fn test_recording_mode_manager_hold_mode() {
        let audio_manager = Arc::new(CrossPlatformAudioManager::new().unwrap());
        let vad = VoiceActivityDetector::new(VadSettings::default());
        let mut manager = RecordingModeManager::new(audio_manager, vad);
        
        manager.set_mode(RecordingMode::Hold);
        
        // Press button
        manager.on_button_press().await.unwrap();
        assert!(manager.is_recording());
        
        // Release button
        let result = manager.on_button_release().await.unwrap();
        assert!(!manager.is_recording());
        assert!(result.is_some());
    }
}
