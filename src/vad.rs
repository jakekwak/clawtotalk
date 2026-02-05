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
/// 
/// Implements real-time voice activity detection using energy-based analysis
/// with adaptive thresholding to distinguish speech from background noise.
pub struct VoiceActivityDetector {
    threshold: f32,
    window_size: usize,
    silence_duration: Duration,
    last_speech_time: Option<Instant>,
    energy_buffer: Vec<f32>,
    noise_floor: f32,
    noise_floor_samples: usize,
}

impl VoiceActivityDetector {
    pub fn new(settings: VadSettings) -> Self {
        Self {
            threshold: settings.threshold,
            window_size: settings.window_size,
            silence_duration: Duration::from_millis(settings.silence_duration_ms),
            last_speech_time: None,
            energy_buffer: Vec::with_capacity(settings.window_size),
            noise_floor: 0.0,
            noise_floor_samples: 0,
        }
    }
    
    /// Analyze an audio frame and return VAD result
    /// 
    /// This method performs real-time analysis of audio frames to detect speech activity.
    /// It uses energy-based detection with adaptive noise floor estimation to distinguish
    /// between speech and background noise.
    pub fn analyze_frame(&mut self, audio_frame: &[f32]) -> VadResult {
        if audio_frame.is_empty() {
            return VadResult::Unknown;
        }
        
        // Calculate energy of the frame
        let energy = self.calculate_energy(audio_frame);
        
        // Update noise floor estimation (adaptive)
        self.update_noise_floor(energy);
        
        // Add to buffer
        self.energy_buffer.push(energy);
        if self.energy_buffer.len() > self.window_size {
            self.energy_buffer.remove(0);
        }
        
        // Calculate average energy over the window
        let avg_energy: f32 = self.energy_buffer.iter().sum::<f32>() / self.energy_buffer.len() as f32;
        
        // Adaptive threshold based on noise floor
        let adaptive_threshold = self.noise_floor + self.threshold;
        
        // Determine if speech is detected
        if avg_energy > adaptive_threshold {
            self.last_speech_time = Some(Instant::now());
            VadResult::Speech
        } else if let Some(last_time) = self.last_speech_time {
            // Continue detecting speech for silence_duration after last detection
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
        self.noise_floor = 0.0;
        self.noise_floor_samples = 0;
    }
    
    /// Get the current noise floor estimate
    pub fn get_noise_floor(&self) -> f32 {
        self.noise_floor
    }
    
    /// Get the current adaptive threshold
    pub fn get_adaptive_threshold(&self) -> f32 {
        self.noise_floor + self.threshold
    }
    
    /// Calculate energy of audio frame using RMS (Root Mean Square)
    fn calculate_energy(&self, frame: &[f32]) -> f32 {
        if frame.is_empty() {
            return 0.0;
        }
        
        let sum_squares: f32 = frame.iter().map(|&x| x * x).sum();
        (sum_squares / frame.len() as f32).sqrt()
    }
    
    /// Update noise floor estimation using exponential moving average
    /// 
    /// This helps the VAD adapt to changing background noise levels.
    /// We only update the noise floor when energy is below the current threshold
    /// to avoid including speech in the noise estimate.
    fn update_noise_floor(&mut self, energy: f32) {
        const MAX_NOISE_SAMPLES: usize = 100;
        const ALPHA: f32 = 0.95; // Smoothing factor for exponential moving average
        
        // Only update noise floor if energy is low (likely background noise)
        if energy < self.noise_floor + self.threshold || self.noise_floor_samples < 10 {
            if self.noise_floor_samples == 0 {
                self.noise_floor = energy;
            } else {
                // Exponential moving average
                self.noise_floor = ALPHA * self.noise_floor + (1.0 - ALPHA) * energy;
            }
            
            self.noise_floor_samples = (self.noise_floor_samples + 1).min(MAX_NOISE_SAMPLES);
        }
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
        assert_eq!(vad.get_noise_floor(), 0.0);
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
        
        // Feed a few frames to establish baseline
        for _ in 0..5 {
            let low_frame = vec![0.001; 1024];
            vad.analyze_frame(&low_frame);
        }
        
        // Now feed speech (significantly above threshold)
        let speech_frame = vec![0.5; 1024]; // Well above default threshold
        
        let result = vad.analyze_frame(&speech_frame);
        assert_eq!(result, VadResult::Speech);
        assert!(vad.is_speech_detected());
    }
    
    #[test]
    fn test_vad_reset() {
        let mut vad = VoiceActivityDetector::default();
        
        // Feed a few frames to establish baseline
        for _ in 0..5 {
            let low_frame = vec![0.001; 1024];
            vad.analyze_frame(&low_frame);
        }
        
        // Feed speech
        let speech_frame = vec![0.5; 1024];
        vad.analyze_frame(&speech_frame);
        assert!(vad.is_speech_detected());
        
        vad.reset();
        assert!(!vad.is_speech_detected());
        assert_eq!(vad.get_noise_floor(), 0.0);
    }
    
    #[test]
    fn test_vad_empty_frame() {
        let mut vad = VoiceActivityDetector::default();
        let empty_frame: Vec<f32> = vec![];
        
        let result = vad.analyze_frame(&empty_frame);
        assert_eq!(result, VadResult::Unknown);
    }
    
    #[test]
    fn test_vad_noise_floor_adaptation() {
        let mut vad = VoiceActivityDetector::default();
        
        // Feed some low-energy frames to establish noise floor
        for _ in 0..10 {
            let noise_frame = vec![0.001; 1024];
            vad.analyze_frame(&noise_frame);
        }
        
        let noise_floor = vad.get_noise_floor();
        assert!(noise_floor > 0.0, "Noise floor should be established");
        assert!(noise_floor < 0.01, "Noise floor should be low for quiet noise");
    }
    
    #[test]
    fn test_vad_distinguishes_speech_from_noise() {
        let mut vad = VoiceActivityDetector::default();
        
        // Establish noise floor with background noise
        for _ in 0..10 {
            let noise_frame = vec![0.005; 1024];
            vad.analyze_frame(&noise_frame);
        }
        
        // Now introduce speech (significantly higher energy than noise + threshold)
        let speech_frame = vec![0.5; 1024]; // Much higher than noise
        let result = vad.analyze_frame(&speech_frame);
        
        assert_eq!(result, VadResult::Speech);
        assert!(vad.is_speech_detected());
    }
    
    #[test]
    fn test_vad_silence_duration() {
        use std::thread;
        
        let settings = VadSettings {
            threshold: 0.02,
            window_size: 1024,
            silence_duration_ms: 100, // Short duration for testing
        };
        let mut vad = VoiceActivityDetector::new(settings);
        
        // Establish baseline
        for _ in 0..5 {
            let low_frame = vec![0.001; 1024];
            vad.analyze_frame(&low_frame);
        }
        
        // Detect speech
        let speech_frame = vec![0.5; 1024];
        vad.analyze_frame(&speech_frame);
        assert!(vad.is_speech_detected());
        
        // Feed silence
        let silent_frame = vec![0.001; 1024];
        vad.analyze_frame(&silent_frame);
        
        // Should still detect speech within silence duration
        assert!(vad.is_speech_detected());
        
        // Wait for silence duration to expire
        thread::sleep(Duration::from_millis(150));
        
        // Should no longer detect speech
        assert!(!vad.is_speech_detected());
    }
}
