use std::time::{Duration, Instant};
use crate::models::VadSettings;

/// Voice Activity Detection result
#[derive(Debug, Clone, PartialEq)]
pub enum VadResult {
    Speech,
    Silence,
    Unknown,
}

/// Voice Activity Detector
pub struct VoiceActivityDetector {
    threshold: f32,
    window_size: usize,
    silence_duration: Duration,
    last_speech_time: Option<Instant>,
    energy_buffer: Vec<f32>,
}

impl VoiceActivityDetector {
    pub fn new(settings: VadSettings) -> Self {
        Self {
            threshold: settings.threshold,
            window_size: settings.window_size,
            silence_duration: Duration::from_millis(settings.silence_duration_ms),
            last_speech_time: None,
            energy_buffer: Vec::with_capacity(settings.window_size),
        }
    }
    
    /// Analyze an audio frame and return VAD result
    pub fn analyze_frame(&mut self, audio_frame: &[f32]) -> VadResult {
        // Calculate energy of the frame
        let energy = self.calculate_energy(audio_frame);
        
        // Add to buffer
        self.energy_buffer.push(energy);
        if self.energy_buffer.len() > self.window_size {
            self.energy_buffer.remove(0);
        }
        
        // Calculate average energy
        let avg_energy: f32 = self.energy_buffer.iter().sum::<f32>() / self.energy_buffer.len() as f32;
        
        // Determine if speech is detected
        if avg_energy > self.threshold {
            self.last_speech_time = Some(Instant::now());
            VadResult::Speech
        } else if let Some(last_time) = self.last_speech_time {
            if last_time.elapsed() < self.silence_duration {
                VadResult::Speech
            } else {
                VadResult::Silence
            }
        } else {
            VadResult::Silence
        }
    }
    
    /// Check if speech is currently detected
    pub fn is_speech_detected(&self) -> bool {
        if let Some(last_time) = self.last_speech_time {
            last_time.elapsed() < self.silence_duration
        } else {
            false
        }
    }
    
    /// Reset the detector state
    pub fn reset(&mut self) {
        self.last_speech_time = None;
        self.energy_buffer.clear();
    }
    
    /// Calculate energy of audio frame
    fn calculate_energy(&self, frame: &[f32]) -> f32 {
        if frame.is_empty() {
            return 0.0;
        }
        
        let sum_squares: f32 = frame.iter().map(|&x| x * x).sum();
        (sum_squares / frame.len() as f32).sqrt()
    }
}

impl Default for VoiceActivityDetector {
    fn default() -> Self {
        Self::new(VadSettings::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vad_creation() {
        let vad = VoiceActivityDetector::default();
        assert!(!vad.is_speech_detected());
    }
    
    #[test]
    fn test_vad_silence_detection() {
        let mut vad = VoiceActivityDetector::default();
        let silent_frame = vec![0.0; 1024];
        
        let result = vad.analyze_frame(&silent_frame);
        assert_eq!(result, VadResult::Silence);
    }
    
    #[test]
    fn test_vad_speech_detection() {
        let mut vad = VoiceActivityDetector::default();
        let speech_frame = vec![0.1; 1024]; // Above default threshold
        
        let result = vad.analyze_frame(&speech_frame);
        assert_eq!(result, VadResult::Speech);
        assert!(vad.is_speech_detected());
    }
    
    #[test]
    fn test_vad_reset() {
        let mut vad = VoiceActivityDetector::default();
        let speech_frame = vec![0.1; 1024];
        
        vad.analyze_frame(&speech_frame);
        assert!(vad.is_speech_detected());
        
        vad.reset();
        assert!(!vad.is_speech_detected());
    }
}
