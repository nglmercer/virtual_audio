//! # Virtual Audio Cable (VAC)
//!
//! Una implementación multiplataforma de cable de audio virtual en Rust.
//! Esta librería permite crear dispositivos de audio virtuales para
//! enrutar audio entre diferentes aplicaciones.
//!
//! ## Características
//!
//! - **Baja latencia**: Optimizado para un retraso mínimo de audio.
//! - **Multiplataforma**: Soporte para Linux (PulseAudio/PipeWire) y Windows.
//! - **Seguro en tiempo real**: Diseñado para cumplir con las restricciones de callbacks de audio.
//! - **Memoria segura**: Aprovecha las garantías de seguridad de Rust.
//!
//! ## Arquitectura
//!
//! La librería se organiza en varios módulos fundamentales:
//!
//! - `buffer`: Gestión de buffers circulares para transferencia de datos.
//! - `audio`: Procesamiento de audio, remuestreo y conversión de formato.
//! - `platform`: Implementaciones específicas para cada sistema operativo.
//!
//! ## Soporte de Plataformas
//!
//! ### Linux
//! - Utiliza PulseAudio/PipeWire para el enrutamiento.
//! - Implementación en espacio de usuario extremadamente flexible.
//! - Soporta enrutamiento por aplicación y global.
//!
//! ### Windows
//! - Basado en el modelo de controladores WDM/WaveRT.
//! - Soporte para Windows 10/11.
//!
//! ## Example
//!
//! ```rust,no_run
//! use virtual_audio_cable::{VirtualCable, AudioFormat, CableConfig, VirtualCableTrait};
//!
//! fn main() -> anyhow::Result<()> {
//!     let config = CableConfig::default();
//!     let mut cable = VirtualCable::new(config)?;
//!     cable.start()?;
//!     
//!     // List applications playing audio
//!     let apps = cable.list_applications()?;
//!     for app in apps {
//!         println!("Found app: {}", app.name);
//!     }
//!
//!     // Route a specific application to the virtual cable
//!     // cable.route_application("123")?;
//!
//!     // Route global system audio
//!     cable.route_system_audio()?;
//!     
//!     cable.stop()?;
//!     Ok(())
//! }
//! ```

// Re-export public modules
pub mod buffer;
pub mod audio;

// Platform-specific module
mod platform;
pub use platform::{VirtualCable, VirtualCableTrait, AudioApplication, AudioOutput};

// Common error types
pub use crate::audio::AudioProcessor;
pub use crate::buffer::{RingBuffer, TripleRingBuffer};

use thiserror::Error;

/// Result type for the library
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for virtual audio cable operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("Buffer error: {0}")]
    BufferError(String),
    
    #[error("Audio processing error: {0}")]
    AudioError(String),
    
    #[error("Platform error: {0}")]
    PlatformError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

/// Configuration for the virtual audio cable
#[derive(Debug, Clone)]
pub struct CableConfig {
    /// Sample rate in Hz (e.g., 44100, 48000, 96000)
    pub sample_rate: u32,
    
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    
    /// Buffer size in samples (typically 256-4096)
    pub buffer_size: usize,
    
    /// Audio format (F32, S16, etc.)
    pub format: AudioFormat,
    
    /// Device name for the virtual cable
    pub device_name: String,
}

impl Default for CableConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 1024,
            format: AudioFormat::F32LE,
            device_name: "Virtual Audio Cable".to_string(),
        }
    }
}

/// Audio format specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// 32-bit floating point, little-endian
    F32LE,
    
    /// 16-bit signed integer, little-endian
    S16LE,
    
    /// 24-bit signed integer, little-endian
    S24LE,
    
    /// 32-bit signed integer, little-endian
    S32LE,
}

impl AudioFormat {
    /// Returns the number of bytes per sample for this format
    pub fn bytes_per_sample(&self) -> usize {
        match self {
            AudioFormat::F32LE => 4,
            AudioFormat::S16LE => 2,
            AudioFormat::S24LE => 3,
            AudioFormat::S32LE => 4,
        }
    }
    
    /// Returns a human-readable name for the format
    pub fn name(&self) -> &'static str {
        match self {
            AudioFormat::F32LE => "F32LE",
            AudioFormat::S16LE => "S16LE",
            AudioFormat::S24LE => "S24LE",
            AudioFormat::S32LE => "S32LE",
        }
    }
}
