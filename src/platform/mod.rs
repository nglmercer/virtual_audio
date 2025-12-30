//! Platform-specific implementations for virtual audio cable.
//!
//! This module provides different implementations for different operating systems:
//! - Linux: Uses PipeWire for user-space audio routing
//! - Windows: Uses WDM/WaveRT kernel driver

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
        pub use linux::LinuxVirtualCable as VirtualCable;
    } else if #[cfg(windows)] {
        mod windows;
        pub use windows::WindowsVirtualCable as VirtualCable;
    } else {
        compile_error!("Unsupported platform. Only Linux and Windows are currently supported.");
    }
}

use crate::{CableConfig, Error};

/// Información detallada sobre una aplicación que está emitiendo audio en el sistema.
#[derive(Debug, Clone)]
pub struct AudioApplication {
    /// ID interno del flujo de audio (por ejemplo, el índice del sink-input en PulseAudio).
    pub id: String,
    /// Nombre amigable de la aplicación (ej. "Discord", "Firefox").
    pub name: String,
    /// ID del proceso (PID) si está disponible.
    pub pid: Option<u32>,
    /// Identificador único de aplicación (ej. "com.discordapp.Discord").
    pub app_id: Option<String>,
}

/// Información sobre un dispositivo de salida de audio físico o virtual.
#[derive(Debug, Clone)]
pub struct AudioOutput {
    /// Nombre interno del dispositivo (ej. "alsa_output.pci-0000_00_1f.3.analog-stereo").
    pub name: String,
    /// Descripción amigable (ej. "Altavoces internos").
    pub description: String,
    /// Si es el dispositivo por defecto actualmente.
    pub is_default: bool,
}

/// Definición de la interfaz para implementaciones de cables de audio virtuales por plataforma.
pub trait VirtualCableTrait: Send + Sync {
    /// Crea un nuevo cable virtual con la configuración dada.
    fn new(config: CableConfig) -> Result<Self, Error>
    where
        Self: Sized;

    /// Inicia el cable virtual.
    fn start(&mut self) -> Result<(), Error>;

    /// Detiene el cable virtual.
    fn stop(&mut self) -> Result<(), Error>;

    /// Devuelve verdadero si el cable está en ejecución.
    fn is_running(&self) -> bool;

    /// Obtiene las estadísticas actuales del cable.
    fn get_stats(&self) -> CableStats;

    /// Lista todas las aplicaciones que están reproduciendo audio actualmente y pueden ser enrutadas.
    fn list_applications(&self) -> Result<Vec<AudioApplication>, Error>;

    /// Enruta el audio de una aplicación específica hacia el cable virtual.
    fn route_application(&self, app_id: &str) -> Result<(), Error>;

    /// Enruta todo el audio del sistema (global) hacia el cable virtual.
    fn route_system_audio(&self) -> Result<(), Error>;

    /// Desenlaza una aplicación del cable virtual, devolviendo su audio al dispositivo por defecto.
    fn unroute_application(&self, app_id: &str) -> Result<(), Error>;

    /// Lista todos los dispositivos de salida de audio disponibles.
    fn list_outputs(&self) -> Result<Vec<AudioOutput>, Error>;

    /// Duplica el audio de una salida hacia otra.
    ///
    /// # Argumentos
    /// * `source_name` - Nombre del dispositivo de origen.
    /// * `target_name` - Nombre del dispositivo de destino.
    fn duplicate_output(&self, source_name: &str, target_name: &str) -> Result<(), Error>;

    /// Detiene todas las duplicaciones activas.
    fn stop_all_duplications(&self) -> Result<(), Error>;
}

/// Statistics about the virtual cable operation.
#[derive(Debug, Clone)]
pub struct CableStats {
    /// Whether the cable is currently active
    pub is_running: bool,

    /// Number of samples processed
    pub samples_processed: u64,

    /// Number of underruns (buffer underflow events)
    pub underruns: u64,

    /// Number of overruns (buffer overflow events)
    pub overruns: u64,

    /// Current latency in milliseconds
    pub latency_ms: f64,

    /// CPU usage percentage (0.0-100.0)
    pub cpu_usage: f64,
}

impl Default for CableStats {
    fn default() -> Self {
        Self {
            is_running: false,
            samples_processed: 0,
            underruns: 0,
            overruns: 0,
            latency_ms: 0.0,
            cpu_usage: 0.0,
        }
    }
}
