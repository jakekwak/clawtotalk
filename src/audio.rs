use async_trait::async_trait;
use crate::error::AudioError;
use crate::models::AudioSettings;
use crate::platform;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use std::sync::{Arc, Mutex};

/// Audio manager trait for cross-platform audio handling
#[async_trait]
pub trait AudioManager: Send + Sync {
    /// Start recording audio
    async fn start_recording(&self) -> Result<(), AudioError>;
    
    /// Stop recording and return audio data
    async fn stop_recording(&self) -> Result<Vec<u8>, AudioError>;
    
    /// Play audio data
    async fn play_audio(&self, data: &[u8]) -> Result<(), AudioError>;
    
    /// Get current audio level (0.0 to 1.0)
    fn get_audio_level(&self) -> f32;
    
    /// Request platform-specific permissions
    fn request_permissions(&self) -> Result<(), AudioError>;
}

/// Shared audio buffer for recording
#[derive(Clone)]
struct AudioBuffer {
    data: Arc<Mutex<Vec<f32>>>,
    level: Arc<Mutex<f32>>,
    is_recording: Arc<Mutex<bool>>,
}

impl AudioBuffer {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(Vec::new())),
            level: Arc::new(Mutex::new(0.0)),
            is_recording: Arc::new(Mutex::new(false)),
        }
    }
    
    fn append(&self, samples: &[f32]) {
        if *self.is_recording.lock().unwrap() {
            if let Ok(mut data) = self.data.lock() {
                data.extend_from_slice(samples);
            }
            
            // Update audio level
            if !samples.is_empty() {
                let rms = (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt();
                if let Ok(mut level) = self.level.lock() {
                    *level = rms;
                }
            }
        }
    }
    
    fn get_data(&self) -> Vec<f32> {
        self.data.lock().unwrap().clone()
    }
    
    fn clear(&self) {
        if let Ok(mut data) = self.data.lock() {
            data.clear();
        }
        if let Ok(mut level) = self.level.lock() {
            *level = 0.0;
        }
    }
    
    fn get_level(&self) -> f32 {
        self.level.lock().unwrap().clone()
    }
    
    fn start_recording(&self) {
        *self.is_recording.lock().unwrap() = true;
    }
    
    fn stop_recording(&self) {
        *self.is_recording.lock().unwrap() = false;
    }
}

/// Cross-platform audio manager implementation
pub struct CrossPlatformAudioManager {
    #[allow(dead_code)] // Used in platform-specific code
    settings: AudioSettings,
    recording_buffer: AudioBuffer,
    #[cfg(target_os = "windows")]
    windows_optimizer: Option<crate::platform::windows::WindowsAudioOptimizer>,
}

impl CrossPlatformAudioManager {
    pub fn new() -> Result<Self, AudioError> {
        Self::with_settings(AudioSettings::default())
    }
    
    pub fn with_settings(settings: AudioSettings) -> Result<Self, AudioError> {
        // Verify audio devices are available
        let host = cpal::default_host();
        
        let _input_device = host
            .default_input_device()
            .ok_or(AudioError::DeviceNotFound)?;
        
        let _output_device = host
            .default_output_device()
            .ok_or(AudioError::DeviceNotFound)?;
        
        log::info!("Audio devices verified");
        
        #[cfg(target_os = "windows")]
        {
            let mut optimizer = crate::platform::windows::WindowsAudioOptimizer::new();
            
            // Apply Windows-specific optimizations
            if let Err(e) = optimizer.apply_thread_optimizations() {
                log::warn!("Failed to apply Windows thread optimizations: {}", e);
            }
            
            Ok(Self {
                settings,
                recording_buffer: AudioBuffer::new(),
                windows_optimizer: Some(optimizer),
            })
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Ok(Self {
                settings,
                recording_buffer: AudioBuffer::new(),
            })
        }
    }
    
    /// Enable low-latency mode (Windows only)
    #[cfg(target_os = "windows")]
    pub fn enable_low_latency(&mut self) -> Result<(), AudioError> {
        if let Some(optimizer) = &mut self.windows_optimizer {
            optimizer.enable_exclusive_mode()?;
            let buffer_size = crate::platform::windows::WindowsAudioOptimizer::get_minimum_buffer_size(self.settings.sample_rate);
            optimizer.set_buffer_size(buffer_size);
            log::info!("Low-latency mode enabled");
        }
        Ok(())
    }
    
    /// Get Windows audio configuration
    #[cfg(target_os = "windows")]
    pub fn get_windows_config(&self) -> Option<crate::platform::windows::WindowsAudioConfig> {
        self.windows_optimizer.as_ref().map(|opt| opt.get_config())
    }
}

impl Default for CrossPlatformAudioManager {
    fn default() -> Self {
        Self::new().expect("Failed to create audio manager")
    }
}

#[async_trait]
impl AudioManager for CrossPlatformAudioManager {
    async fn start_recording(&self) -> Result<(), AudioError> {
        // Clear previous recording
        self.recording_buffer.clear();
        self.recording_buffer.start_recording();
        
        let host = cpal::default_host();
        let input_device = host
            .default_input_device()
            .ok_or(AudioError::DeviceNotFound)?;
        
        // Get input config
        let config = input_device
            .default_input_config()
            .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
        
        log::info!("Starting recording with config: {:?}", config);
        
        // Create recording buffer clone for the stream
        let buffer = self.recording_buffer.clone();
        
        // Build and start input stream
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                Self::build_input_stream::<f32>(&input_device, &config.into(), buffer)?
            }
            cpal::SampleFormat::I16 => {
                Self::build_input_stream::<i16>(&input_device, &config.into(), buffer)?
            }
            cpal::SampleFormat::U16 => {
                Self::build_input_stream::<u16>(&input_device, &config.into(), buffer)?
            }
            _ => return Err(AudioError::UnsupportedFormat),
        };
        
        // Start the stream
        stream.play()
            .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
        
        // Keep stream alive by leaking it (will be cleaned up on stop)
        std::mem::forget(stream);
        
        log::info!("Recording started successfully");
        Ok(())
    }
    
    async fn stop_recording(&self) -> Result<Vec<u8>, AudioError> {
        // Stop recording
        self.recording_buffer.stop_recording();
        
        // Small delay to ensure last samples are captured
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Get recorded data
        let samples = self.recording_buffer.get_data();
        
        log::info!("Recording stopped, captured {} samples", samples.len());
        
        // Convert f32 samples to i16 PCM for compatibility
        let pcm_data: Vec<i16> = samples.iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .collect();
        
        // Convert to bytes
        let mut bytes = Vec::with_capacity(pcm_data.len() * 2);
        for sample in pcm_data {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        
        Ok(bytes)
    }
    
    async fn play_audio(&self, data: &[u8]) -> Result<(), AudioError> {
        if data.is_empty() {
            return Err(AudioError::UnsupportedFormat);
        }
        
        // Convert bytes to i16 samples
        let mut samples = Vec::with_capacity(data.len() / 2);
        for chunk in data.chunks_exact(2) {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            samples.push(sample);
        }
        
        log::info!("Playing audio: {} samples", samples.len());
        
        let host = cpal::default_host();
        let output_device = host
            .default_output_device()
            .ok_or(AudioError::DeviceNotFound)?;
        
        // Get output config
        let config = output_device
            .default_output_config()
            .map_err(|e| AudioError::PlaybackFailed(e.to_string()))?;
        
        // Create playback stream
        let samples = Arc::new(Mutex::new(samples));
        let sample_index = Arc::new(Mutex::new(0usize));
        let total_samples = samples.lock().unwrap().len();
        
        let samples_clone = samples.clone();
        let index_clone = sample_index.clone();
        
        {
            let stream = match config.sample_format() {
                cpal::SampleFormat::F32 => {
                    Self::build_output_stream::<f32>(&output_device, &config.into(), samples_clone, index_clone)?
                }
                cpal::SampleFormat::I16 => {
                    Self::build_output_stream::<i16>(&output_device, &config.into(), samples_clone, index_clone)?
                }
                cpal::SampleFormat::U16 => {
                    Self::build_output_stream::<u16>(&output_device, &config.into(), samples_clone, index_clone)?
                }
                _ => return Err(AudioError::UnsupportedFormat),
            };
            
            // Play the stream
            stream.play()
                .map_err(|e| AudioError::PlaybackFailed(e.to_string()))?;
            
            // Keep stream alive by leaking it
            std::mem::forget(stream);
        }
        
        // Wait for playback to complete
        while *sample_index.lock().unwrap() < total_samples {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        log::info!("Audio playback completed");
        Ok(())
    }
    
    fn get_audio_level(&self) -> f32 {
        self.recording_buffer.get_level()
    }
    
    fn request_permissions(&self) -> Result<(), AudioError> {
        // Use platform-specific permission handling
        platform::request_audio_permissions()
    }
}

impl CrossPlatformAudioManager {
    fn build_input_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        buffer: AudioBuffer,
    ) -> Result<cpal::Stream, AudioError>
    where
        T: SizedSample,
        f32: FromSample<T>,
    {
        let err_fn = |err| {
            log::error!("Stream error: {}", err);
        };
        
        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    let samples: Vec<f32> = data.iter()
                        .map(|&s| s.to_sample::<f32>())
                        .collect();
                    buffer.append(&samples);
                },
                err_fn,
                None,
            )
            .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
        
        Ok(stream)
    }
    
    fn build_output_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        samples: Arc<Mutex<Vec<i16>>>,
        sample_index: Arc<Mutex<usize>>,
    ) -> Result<cpal::Stream, AudioError>
    where
        T: SizedSample + FromSample<f32>,
    {
        let err_fn = |err| {
            log::error!("Stream error: {}", err);
        };
        
        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    let samples = samples.lock().unwrap();
                    let mut index = sample_index.lock().unwrap();
                    
                    for frame in data.iter_mut() {
                        if *index < samples.len() {
                            let sample = samples[*index] as f32 / i16::MAX as f32;
                            *frame = T::from_sample(sample);
                            *index += 1;
                        } else {
                            *frame = T::from_sample(0.0f32);
                        }
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| AudioError::PlaybackFailed(e.to_string()))?;
        
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_manager_creation() {
        let manager = CrossPlatformAudioManager::new();
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_audio_buffer() {
        let buffer = AudioBuffer::new();
        
        // Start recording
        buffer.start_recording();
        
        // Test append
        let samples = vec![0.1, 0.2, 0.3];
        buffer.append(&samples);
        
        let data = buffer.get_data();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], 0.1);
        
        // Test level
        let level = buffer.get_level();
        assert!(level > 0.0);
        
        // Test clear
        buffer.clear();
        assert_eq!(buffer.get_data().len(), 0);
        assert_eq!(buffer.get_level(), 0.0);
    }
    
    #[tokio::test]
    async fn test_empty_audio_handling() {
        let manager = CrossPlatformAudioManager::new().unwrap();
        let result = manager.play_audio(&[]).await;
        assert!(matches!(result, Err(AudioError::UnsupportedFormat)));
    }
}
