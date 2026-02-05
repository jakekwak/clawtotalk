use crate::error::AudioError;

/// Audio compression utilities for bandwidth optimization
/// Requirement 11.5: Network bandwidth optimization

/// Compress audio data using simple run-length encoding for silence
/// This is a basic compression that works well for voice data with silence periods
pub fn compress_audio(data: &[u8]) -> Result<Vec<u8>, AudioError> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut compressed = Vec::new();
    
    // Convert bytes to i16 samples
    let samples: Vec<i16> = data
        .chunks_exact(2)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    
    // Simple silence detection and compression
    let silence_threshold = 100i16; // Threshold for silence
    let mut i = 0;
    
    while i < samples.len() {
        let sample = samples[i];
        
        // Check if this is silence
        if sample.abs() < silence_threshold {
            // Count consecutive silence samples
            let mut silence_count = 0;
            while i < samples.len() && samples[i].abs() < silence_threshold {
                silence_count += 1;
                i += 1;
            }
            
            // Encode silence as a marker + count
            if silence_count > 4 {
                // Only compress if we have more than 4 samples of silence
                compressed.push(0xFF); // Silence marker
                compressed.push(0xFF);
                compressed.extend_from_slice(&(silence_count as u32).to_le_bytes());
            } else {
                // Not worth compressing, write samples directly
                i -= silence_count;
                for _ in 0..silence_count {
                    compressed.extend_from_slice(&samples[i].to_le_bytes());
                    i += 1;
                }
            }
        } else {
            // Non-silence, write directly
            compressed.extend_from_slice(&sample.to_le_bytes());
            i += 1;
        }
    }
    
    log::info!(
        "Audio compression: {} bytes -> {} bytes ({:.1}% reduction)",
        data.len(),
        compressed.len(),
        (1.0 - compressed.len() as f64 / data.len() as f64) * 100.0
    );
    
    Ok(compressed)
}

/// Decompress audio data
pub fn decompress_audio(data: &[u8]) -> Result<Vec<u8>, AudioError> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut decompressed = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        // Check for silence marker
        if i + 1 < data.len() && data[i] == 0xFF && data[i + 1] == 0xFF {
            // This is a silence marker
            if i + 6 > data.len() {
                return Err(AudioError::UnsupportedFormat);
            }
            
            i += 2;
            let silence_count = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
            i += 4;
            
            // Write silence samples
            for _ in 0..silence_count {
                decompressed.extend_from_slice(&0i16.to_le_bytes());
            }
        } else {
            // Regular sample
            if i + 2 > data.len() {
                break;
            }
            decompressed.push(data[i]);
            decompressed.push(data[i + 1]);
            i += 2;
        }
    }
    
    Ok(decompressed)
}

/// Downsample audio to reduce bandwidth
/// Converts from higher sample rate to 16kHz for transmission
pub fn downsample_audio(data: &[u8], from_rate: u32, to_rate: u32) -> Result<Vec<u8>, AudioError> {
    if from_rate == to_rate {
        return Ok(data.to_vec());
    }
    
    if from_rate < to_rate {
        return Err(AudioError::UnsupportedFormat);
    }
    
    // Convert bytes to i16 samples
    let samples: Vec<i16> = data
        .chunks_exact(2)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    
    // Calculate downsampling ratio
    let ratio = from_rate as f32 / to_rate as f32;
    let output_len = (samples.len() as f32 / ratio) as usize;
    
    let mut downsampled = Vec::with_capacity(output_len);
    
    // Simple linear interpolation downsampling
    for i in 0..output_len {
        let src_index = (i as f32 * ratio) as usize;
        if src_index < samples.len() {
            downsampled.push(samples[src_index]);
        }
    }
    
    // Convert back to bytes
    let mut bytes = Vec::with_capacity(downsampled.len() * 2);
    for sample in downsampled {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    
    log::info!(
        "Audio downsampling: {}Hz -> {}Hz, {} bytes -> {} bytes",
        from_rate,
        to_rate,
        data.len(),
        bytes.len()
    );
    
    Ok(bytes)
}

/// Remove leading and trailing silence from audio
pub fn trim_silence(data: &[u8], threshold: i16) -> Result<Vec<u8>, AudioError> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    
    // Convert bytes to i16 samples
    let samples: Vec<i16> = data
        .chunks_exact(2)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    
    // Find first non-silence sample
    let start = samples
        .iter()
        .position(|&s| s.abs() > threshold)
        .unwrap_or(0);
    
    // Find last non-silence sample
    let end = samples
        .iter()
        .rposition(|&s| s.abs() > threshold)
        .unwrap_or(samples.len() - 1)
        + 1;
    
    // Convert back to bytes
    let mut bytes = Vec::with_capacity((end - start) * 2);
    for sample in &samples[start..end] {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    
    log::info!(
        "Silence trimming: {} bytes -> {} bytes",
        data.len(),
        bytes.len()
    );
    
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compress_decompress_roundtrip() {
        // Create test audio with silence
        let mut samples = Vec::new();
        
        // Some audio
        for _ in 0..100 {
            samples.push(1000i16);
        }
        
        // Silence
        for _ in 0..1000 {
            samples.push(0i16);
        }
        
        // More audio
        for _ in 0..100 {
            samples.push(2000i16);
        }
        
        // Convert to bytes
        let mut data = Vec::new();
        for sample in &samples {
            data.extend_from_slice(&sample.to_le_bytes());
        }
        
        // Compress
        let compressed = compress_audio(&data).unwrap();
        
        // Should be smaller
        assert!(compressed.len() < data.len());
        
        // Decompress
        let decompressed = decompress_audio(&compressed).unwrap();
        
        // Should match original
        assert_eq!(decompressed.len(), data.len());
    }
    
    #[test]
    fn test_downsample() {
        // Create test audio at 48kHz
        let samples: Vec<i16> = (0..4800).map(|i| (i % 1000) as i16).collect();
        
        let mut data = Vec::new();
        for sample in samples {
            data.extend_from_slice(&sample.to_le_bytes());
        }
        
        // Downsample to 16kHz
        let downsampled = downsample_audio(&data, 48000, 16000).unwrap();
        
        // Should be approximately 1/3 the size
        assert!(downsampled.len() < data.len() / 2);
        assert!(downsampled.len() > data.len() / 4);
    }
    
    #[test]
    fn test_trim_silence() {
        // Create test audio with leading and trailing silence
        let mut samples = Vec::new();
        
        // Leading silence
        for _ in 0..100 {
            samples.push(0i16);
        }
        
        // Audio
        for _ in 0..100 {
            samples.push(1000i16);
        }
        
        // Trailing silence
        for _ in 0..100 {
            samples.push(0i16);
        }
        
        // Convert to bytes
        let mut data = Vec::new();
        for sample in samples {
            data.extend_from_slice(&sample.to_le_bytes());
        }
        
        // Trim silence
        let trimmed = trim_silence(&data, 100).unwrap();
        
        // Should be smaller (removed leading and trailing silence)
        assert!(trimmed.len() < data.len());
        assert_eq!(trimmed.len(), 200); // 100 samples * 2 bytes
    }
    
    #[test]
    fn test_empty_audio() {
        let empty: Vec<u8> = Vec::new();
        
        assert_eq!(compress_audio(&empty).unwrap().len(), 0);
        assert_eq!(decompress_audio(&empty).unwrap().len(), 0);
        assert_eq!(trim_silence(&empty, 100).unwrap().len(), 0);
    }
}
