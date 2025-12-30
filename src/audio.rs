//! Audio processing and resampling functionality.
//!
//! This module provides audio processing capabilities including
//! resampling, format conversion, and audio effects.

use crate::AudioFormat;
use crate::Error;

/// Audio processor for handling sample rate conversion and format conversion.
pub struct AudioProcessor {
    /// Input sample rate
    pub input_sample_rate: u32,
    
    /// Output sample rate
    pub output_sample_rate: u32,
    
    /// Number of channels
    pub channels: u16,
    
    /// Audio format
    pub format: AudioFormat,
    
    /// Resampling factor (output_rate / input_rate)
    resample_factor: f64,
}

impl AudioProcessor {
    /// Creates a new audio processor.
    ///
    /// # Arguments
    ///
    /// * `input_sample_rate` - Input sample rate in Hz
    /// * `output_sample_rate` - Output sample rate in Hz
    /// * `channels` - Number of audio channels
    /// * `format` - Audio format
    pub fn new(
        input_sample_rate: u32,
        output_sample_rate: u32,
        channels: u16,
        format: AudioFormat,
    ) -> Self {
        let resample_factor = output_sample_rate as f64 / input_sample_rate as f64;
        
        Self {
            input_sample_rate,
            output_sample_rate,
            channels,
            format,
            resample_factor,
        }
    }
    
    /// Processes audio samples.
    ///
    /// This method performs resampling if input and output sample rates differ.
    /// For now, it's a simple pass-through implementation.
    ///
    /// # Arguments
    ///
    /// * `input` - Input audio samples
    /// * `output` - Output buffer for processed samples
    ///
    /// # Returns
    ///
    /// Number of samples written to output
    pub fn process(&self, input: &[f32], output: &mut [f32]) -> Result<usize, Error> {
        let to_process = input.len().min(output.len());
        
        if self.input_sample_rate == self.output_sample_rate {
            // Pass-through when sample rates match
            output[..to_process].copy_from_slice(&input[..to_process]);
            return Ok(to_process);
        }
        
        // Simple resampling (linear interpolation - placeholder)
        // In production, use rubato or similar library
        let output_len = ((input.len() as f64) * self.resample_factor) as usize;
        let actual_output = output_len.min(output.len());
        
        for i in 0..actual_output {
            let src_idx = (i as f64 / self.resample_factor) as usize;
            if src_idx < input.len() {
                output[i] = input[src_idx];
            }
        }
        
        Ok(actual_output)
    }
    
    /// Converts audio samples between formats.
    ///
    /// Currently supports F32 to S16 conversion.
    /// More conversions will be added in the future.
    ///
    /// # Arguments
    ///
    /// * `input` - Input samples in source format (as f32 for now)
    /// * `output_format` - Target format
    ///
    /// # Returns
    ///
    /// Vector of bytes in the target format
    pub fn convert_format(&self, input: &[f32], output_format: AudioFormat) -> Vec<u8> {
        match output_format {
            AudioFormat::F32LE => {
                let bytes: &[u8] = unsafe {
                    std::slice::from_raw_parts(
                        input.as_ptr() as *const u8,
                        input.len() * 4,
                    )
                };
                bytes.to_vec()
            }
            AudioFormat::S16LE => {
                let mut output = Vec::with_capacity(input.len() * 2);
                for &sample in input {
                    let s16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                    output.extend_from_slice(&s16.to_le_bytes());
                }
                output
            }
            AudioFormat::S24LE => {
                let mut output = Vec::with_capacity(input.len() * 3);
                for &sample in input {
                    let s24 = (sample.clamp(-1.0, 1.0) * 8388607.0) as i32;
                    output.extend_from_slice(&s24.to_le_bytes()[..3]);
                }
                output
            }
            AudioFormat::S32LE => {
                let mut output = Vec::with_capacity(input.len() * 4);
                for &sample in input {
                    let s32 = (sample.clamp(-1.0, 1.0) * 2147483647.0) as i32;
                    output.extend_from_slice(&s32.to_le_bytes());
                }
                output
            }
        }
    }
    
    /// Converts bytes to f32 samples.
    ///
    /// # Arguments
    ///
    /// * `input` - Input bytes in the specified format
    /// * `input_format` - Format of the input bytes
    ///
    /// # Returns
    ///
    /// Vector of f32 samples
    pub fn bytes_to_samples(&self, input: &[u8], input_format: AudioFormat) -> Vec<f32> {
        let bytes_per_sample = input_format.bytes_per_sample();
        let num_samples = input.len() / bytes_per_sample;
        
        let mut output = Vec::with_capacity(num_samples);
        
        match input_format {
            AudioFormat::F32LE => {
                let samples: &[f32] = unsafe {
                    std::slice::from_raw_parts(
                        input.as_ptr() as *const f32,
                        num_samples,
                    )
                };
                output.extend_from_slice(samples);
            }
            AudioFormat::S16LE => {
                for i in 0..num_samples {
                    let start = i * 2;
                    let s16 = i16::from_le_bytes([
                        input[start],
                        input[start + 1],
                    ]);
                    output.push(s16 as f32 / 32767.0);
                }
            }
            AudioFormat::S24LE => {
                for i in 0..num_samples {
                    let start = i * 3;
                    let mut bytes = [0u8; 4];
                    bytes[0] = input[start];
                    bytes[1] = input[start + 1];
                    bytes[2] = input[start + 2];
                    let s24 = i32::from_le_bytes(bytes);
                    output.push(s24 as f32 / 8388607.0);
                }
            }
            AudioFormat::S32LE => {
                for i in 0..num_samples {
                    let start = i * 4;
                    let s32 = i32::from_le_bytes([
                        input[start],
                        input[start + 1],
                        input[start + 2],
                        input[start + 3],
                    ]);
                    output.push(s32 as f32 / 2147483647.0);
                }
            }
        }
        
        output
    }
    
    /// Returns true if resampling is needed.
    pub fn needs_resampling(&self) -> bool {
        self.input_sample_rate != self.output_sample_rate
    }
}

impl Default for AudioProcessor {
    fn default() -> Self {
        Self::new(48000, 48000, 2, AudioFormat::F32LE)
    }
}

/// Resampler for sample rate conversion.
///
/// This is a placeholder for integration with rubato library.
#[allow(dead_code)]  // channels will be used with rubato integration
pub struct Resampler {
    input_rate: u32,
    output_rate: u32,
    channels: u16,
}

impl Resampler {
    /// Creates a new resampler.
    pub fn new(input_rate: u32, output_rate: u32, channels: u16) -> Self {
        Self {
            input_rate,
            output_rate,
            channels,
        }
    }
    
    /// Resamples audio from input rate to output rate.
    ///
    /// This is a simple linear interpolation implementation.
    /// In production, use rubato library for high-quality resampling.
    pub fn process(&self, input: &[f32]) -> Result<Vec<f32>, Error> {
        if self.input_rate == self.output_rate {
            return Ok(input.to_vec());
        }
        
        let ratio = self.output_rate as f64 / self.input_rate as f64;
        let output_len = ((input.len() as f64) * ratio) as usize;
        let mut output = Vec::with_capacity(output_len);
        
        for i in 0..output_len {
            let src_idx = (i as f64 / ratio) as usize;
            let frac = (i as f64 / ratio) - src_idx as f64;
            
            if src_idx + 1 < input.len() {
                // Linear interpolation
                let y0 = input[src_idx];
                let y1 = input[src_idx + 1];
                let sample = y0 + (y1 - y0) * (frac as f32);
                output.push(sample);
            } else if src_idx < input.len() {
                output.push(input[src_idx]);
            }
        }
        
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_processor_passthrough() {
        let processor = AudioProcessor::new(48000, 48000, 2, AudioFormat::F32LE);
        let input = vec![1.0, -1.0, 0.5, -0.5];
        let mut output = vec![0.0; 4];
        
        let result = processor.process(&input, &mut output).unwrap();
        assert_eq!(result, 4);
        assert_eq!(output, input);
    }
    
    #[test]
    fn test_format_conversion_f32_to_s16() {
        let processor = AudioProcessor::new(48000, 48000, 2, AudioFormat::F32LE);
        let input = vec![1.0, 0.0, -1.0, 0.5];
        
        let bytes = processor.convert_format(&input, AudioFormat::S16LE);
        assert_eq!(bytes.len(), 8); // 4 samples * 2 bytes each
        
        // Check first sample (1.0 -> 32767)
        let s16 = i16::from_le_bytes([bytes[0], bytes[1]]);
        assert_eq!(s16, 32767);
    }
    
    #[test]
    fn test_bytes_to_samples() {
        let processor = AudioProcessor::new(48000, 48000, 2, AudioFormat::F32LE);
        
        // Create S16LE bytes
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&32767i16.to_le_bytes()); // 1.0
        bytes.extend_from_slice(&(-32767i16).to_le_bytes()); // -1.0
        
        let samples = processor.bytes_to_samples(&bytes, AudioFormat::S16LE);
        assert_eq!(samples.len(), 2);
        assert!((samples[0] - 1.0).abs() < 0.01);
        assert!((samples[1] + 1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_resampler() {
        let resampler = Resampler::new(48000, 96000, 2);
        let input = vec![0.0, 1.0, 0.0, -1.0];
        
        let output = resampler.process(&input).unwrap();
        assert_eq!(output.len(), 8); // Double the samples
    }
}
