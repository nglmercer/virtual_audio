//! Linux implementation using PulseAudio/PipeWire.
//!
//! This module provides a user-space virtual audio cable implementation
//! for Linux systems. It supports both PulseAudio (via pactl) and
//! PipeWire for audio routing.

use crate::audio::AudioProcessor;
use crate::buffer::TripleRingBuffer;
use crate::platform::{CableStats, VirtualCableTrait};
use crate::{CableConfig, Error};

use std::process::Command;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Linux virtual audio cable implementation.
pub struct LinuxVirtualCable {
    config: CableConfig,
    is_running: AtomicBool,
    triple_buffer: Arc<Mutex<TripleRingBuffer>>,
    #[allow(dead_code)]
    audio_processor: AudioProcessor,
    
    // Statistics
    samples_processed: AtomicU64,
    underruns: AtomicU64,
    overruns: AtomicU64,
    
    // PulseAudio state
    null_sink_id: Arc<Mutex<Option<String>>>,
    loopback_id: Arc<Mutex<Option<String>>>,
}

impl VirtualCableTrait for LinuxVirtualCable {
    fn new(config: CableConfig) -> Result<Self, Error> {
        let triple_buffer = Arc::new(Mutex::new(TripleRingBuffer::new(config.buffer_size)));
        
        let audio_processor = AudioProcessor::new(
            config.sample_rate,
            config.sample_rate,
            config.channels,
            config.format,
        );
        
        Ok(Self {
            config,
            is_running: AtomicBool::new(false),
            triple_buffer,
            audio_processor,
            samples_processed: AtomicU64::new(0),
            underruns: AtomicU64::new(0),
            overruns: AtomicU64::new(0),
            null_sink_id: Arc::new(Mutex::new(None)),
            loopback_id: Arc::new(Mutex::new(None)),
        })
    }
    
    fn start(&mut self) -> Result<(), Error> {
        if self.is_running.load(Ordering::Relaxed) {
            return Err(Error::PlatformError(
                "Virtual cable is already running".to_string(),
            ));
        }
        
        log::info!("Starting PulseAudio-compatible virtual audio cable");
        
        // 1. Create the null sink
        let sink_name = self.config.device_name.replace(" ", "_");
        let description = &self.config.device_name;
        
        let output = Command::new("pactl")
            .args([
                "load-module",
                "module-null-sink",
                &format!("sink_name={}", sink_name),
                &format!("sink_properties=device.description=\"{}\"", description),
            ])
            .output()
            .map_err(|e| Error::PlatformError(format!("Failed to execute pactl: {}", e)))?;
            
        if !output.status.success() {
            return Err(Error::PlatformError(format!(
                "Failed to create null sink: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        
        let sink_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        *self.null_sink_id.lock().unwrap() = Some(sink_id.clone());
        
        log::info!("Created virtual sink '{}' (ID: {})", sink_name, sink_id);
        
        // 2. Get the default sink monitor to loopback system audio
        let default_sink_output = Command::new("pactl")
            .arg("get-default-sink")
            .output()
            .map_err(|e| Error::PlatformError(format!("Failed to get default sink: {}", e)))?;
            
        if default_sink_output.status.success() {
            let default_sink = String::from_utf8_lossy(&default_sink_output.stdout).trim().to_string();
            let monitor_source = format!("{}.monitor", default_sink);
            
            log::info!("Routing audio from {} to {}", monitor_source, sink_name);
            
            let loopback_output = Command::new("pactl")
                .args([
                    "load-module",
                    "module-loopback",
                    &format!("source={}", monitor_source),
                    &format!("sink={}", sink_name),
                    "latency_msec=20",
                ])
                .output()
                .map_err(|e| Error::PlatformError(format!("Failed to load loopback: {}", e)))?;
                
            if loopback_output.status.success() {
                let lb_id = String::from_utf8_lossy(&loopback_output.stdout).trim().to_string();
                *self.loopback_id.lock().unwrap() = Some(lb_id.clone());
                log::info!("System audio loopback started (ID: {})", lb_id);
            } else {
                log::warn!("Could not start automatic loopback: {}", String::from_utf8_lossy(&loopback_output.stderr));
            }
        }
        
        self.is_running.store(true, Ordering::Relaxed);
        log::info!("Linux virtual audio cable started successfully via PulseAudio");
        
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), Error> {
        if !self.is_running.load(Ordering::Relaxed) {
            return Err(Error::PlatformError(
                "Virtual cable is not running".to_string(),
            ));
        }
        
        log::info!("Stopping PulseAudio virtual audio cable");
        
        // Remove loopback if it exists
        if let Some(lb_id) = self.loopback_id.lock().unwrap().take() {
            let _ = Command::new("pactl")
                .args(["unload-module", &lb_id])
                .status();
            log::info!("Unloaded loopback module {}", lb_id);
        }
        
        // Remove null sink
        if let Some(sink_id) = self.null_sink_id.lock().unwrap().take() {
            let _ = Command::new("pactl")
                .args(["unload-module", &sink_id])
                .status();
            log::info!("Unloaded null sink module {}", sink_id);
        }
        
        self.is_running.store(false, Ordering::Relaxed);
        log::info!("Linux virtual audio cable stopped");
        
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }
    
    fn get_stats(&self) -> CableStats {
        CableStats {
            is_running: self.is_running(),
            samples_processed: self.samples_processed.load(Ordering::Relaxed),
            underruns: self.underruns.load(Ordering::Relaxed),
            overruns: self.overruns.load(Ordering::Relaxed),
            latency_ms: self.calculate_latency(),
            cpu_usage: self.estimate_cpu_usage(),
        }
    }
}

impl LinuxVirtualCable {
    /// Processes audio (wrapper for triple buffer).
    pub fn process_audio(&self, input: &[f32], output: &mut [f32]) -> Result<usize, Error> {
        if !self.is_running() {
            return Err(Error::PlatformError("Cable not running".into()));
        }
        let processed = self.triple_buffer.lock().unwrap().process(input, output)?;
        self.samples_processed.fetch_add(processed as u64, Ordering::Relaxed);
        Ok(processed)
    }

    fn calculate_latency(&self) -> f64 {
        let stats = self.triple_buffer.lock().unwrap().stats();
        (stats.resample_available as f64 * 1000.0) / self.config.sample_rate as f64
    }
    
    fn estimate_cpu_usage(&self) -> f64 {
        if self.is_running() { 0.5 } else { 0.0 }
    }
}
