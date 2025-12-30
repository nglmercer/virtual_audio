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

/// Trait for platform-specific virtual cable implementations.
pub trait VirtualCableTrait: Send + Sync {
    /// Creates a new virtual cable with the given configuration.
    fn new(config: CableConfig) -> Result<Self, Error>
    where
        Self: Sized;
    
    /// Starts the virtual cable.
    fn start(&mut self) -> Result<(), Error>;
    
    /// Stops the virtual cable.
    fn stop(&mut self) -> Result<(), Error>;
    
    /// Returns true if the cable is currently running.
    fn is_running(&self) -> bool;
    
    /// Gets statistics about the cable's operation.
    fn get_stats(&self) -> CableStats;
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
