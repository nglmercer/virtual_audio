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
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Implementación de cable de audio virtual para Linux.
///
/// Gestiona la creación de dispositivos virtuales mediante PulseAudio (pactl)
/// y permite el enrutamiento dinámico de flujos de audio individuales.
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
    active_loopbacks: Arc<Mutex<Vec<String>>>,
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
            active_loopbacks: Arc::new(Mutex::new(Vec::new())),
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
            let default_sink = String::from_utf8_lossy(&default_sink_output.stdout)
                .trim()
                .to_string();
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
                let lb_id = String::from_utf8_lossy(&loopback_output.stdout)
                    .trim()
                    .to_string();
                self.active_loopbacks.lock().unwrap().push(lb_id.clone());
                log::info!("System audio loopback started (ID: {})", lb_id);
            } else {
                log::warn!(
                    "Could not start automatic loopback: {}",
                    String::from_utf8_lossy(&loopback_output.stderr)
                );
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

        // Remove loopbacks
        let mut loopbacks = self.active_loopbacks.lock().unwrap();
        for lb_id in loopbacks.drain(..) {
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

    /// Obtiene las estadísticas actuales de rendimiento y buffers.
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

    fn list_applications(&self) -> Result<Vec<crate::platform::AudioApplication>, Error> {
        let output = Command::new("pactl")
            .args(["list", "sink-inputs"])
            .output()
            .map_err(|e| Error::PlatformError(format!("Failed to execute pactl: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut apps = Vec::new();
        let mut current_app = None;

        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with("Entrada del destino #") || line.starts_with("Sink Input #") {
                if let Some(app) = current_app.take() {
                    apps.push(app);
                }
                let id = line.split('#').next_back().unwrap_or("").to_string();
                current_app = Some(crate::platform::AudioApplication {
                    id,
                    name: "Unknown".into(),
                    pid: None,
                    app_id: None,
                });
            } else if let Some(ref mut app) = current_app {
                if line.contains("application.name =") {
                    app.name = line
                        .split('=')
                        .next_back()
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"')
                        .to_string();
                } else if line.contains("application.process.id =") {
                    app.pid = line
                        .split('=')
                        .next_back()
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"')
                        .parse()
                        .ok();
                } else if line.contains("pipewire.access.portal.app_id =")
                    || line.contains("application.id =")
                {
                    app.app_id = Some(
                        line.split('=')
                            .next_back()
                            .unwrap_or("")
                            .trim()
                            .trim_matches('"')
                            .to_string(),
                    );
                }
            }
        }

        if let Some(app) = current_app {
            apps.push(app);
        }

        Ok(apps)
    }

    fn route_application(&self, app_id: &str) -> Result<(), Error> {
        let sink_name = self.config.device_name.replace(" ", "_");
        let output = Command::new("pactl")
            .args(["move-sink-input", app_id, &sink_name])
            .output()
            .map_err(|e| Error::PlatformError(format!("Failed to route application: {}", e)))?;

        if !output.status.success() {
            return Err(Error::PlatformError(format!(
                "Failed to route application {}: {}",
                app_id,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        log::info!("Routed application {} to {}", app_id, sink_name);
        Ok(())
    }

    fn route_system_audio(&self) -> Result<(), Error> {
        let sink_name = self.config.device_name.replace(" ", "_");
        let default_sink_output = Command::new("pactl")
            .arg("get-default-sink")
            .output()
            .map_err(|e| Error::PlatformError(format!("Failed to get default sink: {}", e)))?;

        if !default_sink_output.status.success() {
            return Err(Error::PlatformError(
                "Could not determine default sink".into(),
            ));
        }

        let default_sink = String::from_utf8_lossy(&default_sink_output.stdout)
            .trim()
            .to_string();
        let monitor_source = format!("{}.monitor", default_sink);

        let output = Command::new("pactl")
            .args([
                "load-module",
                "module-loopback",
                &format!("source={}", monitor_source),
                &format!("sink={}", sink_name),
                "latency_msec=20",
            ])
            .output()
            .map_err(|e| Error::PlatformError(format!("Failed to load loopback: {}", e)))?;

        if output.status.success() {
            let lb_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            self.active_loopbacks.lock().unwrap().push(lb_id.clone());
            log::info!("System audio loopback started (ID: {})", lb_id);
            Ok(())
        } else {
            Err(Error::PlatformError(format!(
                "Failed to start loopback: {}",
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }

    fn unroute_application(&self, app_id: &str) -> Result<(), Error> {
        let default_sink_output = Command::new("pactl")
            .arg("get-default-sink")
            .output()
            .map_err(|e| Error::PlatformError(format!("Failed to get default sink: {}", e)))?;

        if !default_sink_output.status.success() {
            return Err(Error::PlatformError(
                "Could not determine default sink".into(),
            ));
        }

        let default_sink = String::from_utf8_lossy(&default_sink_output.stdout)
            .trim()
            .to_string();

        let output = Command::new("pactl")
            .args(["move-sink-input", app_id, &default_sink])
            .output()
            .map_err(|e| Error::PlatformError(format!("Failed to unroute application: {}", e)))?;

        if !output.status.success() {
            return Err(Error::PlatformError(format!(
                "Failed to unroute application {}: {}",
                app_id,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        log::info!("Unrouted application {} back to {}", app_id, default_sink);
        Ok(())
    }

    fn list_outputs(&self) -> Result<Vec<crate::platform::AudioOutput>, Error> {
        let output = Command::new("pactl")
            .args(["list", "sinks"])
            .output()
            .map_err(|e| Error::PlatformError(format!("Failed to execute pactl: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut outputs = Vec::new();
        let mut current_output = None;

        let default_sink_output = Command::new("pactl").arg("get-default-sink").output().ok();
        let default_sink =
            default_sink_output.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with("Destino #") || line.starts_with("Sink #") {
                if let Some(out) = current_output.take() {
                    outputs.push(out);
                }
                current_output = Some(crate::platform::AudioOutput {
                    name: String::new(),
                    description: String::new(),
                    is_default: false,
                });
            } else if let Some(ref mut out) = current_output {
                if line.starts_with("Nombre:") || line.starts_with("Name:") {
                    out.name = line.split(':').next_back().unwrap_or("").trim().to_string();
                    if let Some(ref def) = default_sink {
                        out.is_default = out.name == *def;
                    }
                } else if line.starts_with("Descripción:") || line.starts_with("Description:") {
                    out.description = line.split(':').next_back().unwrap_or("").trim().to_string();
                }
            }
        }

        if let Some(out) = current_output {
            outputs.push(out);
        }

        Ok(outputs)
    }

    fn duplicate_output(&self, source_name: &str, target_name: &str) -> Result<(), Error> {
        let monitor_source = format!("{}.monitor", source_name);

        let output = Command::new("pactl")
            .args([
                "load-module",
                "module-loopback",
                &format!("source={}", monitor_source),
                &format!("sink={}", target_name),
                "latency_msec=20",
            ])
            .output()
            .map_err(|e| Error::PlatformError(format!("Failed to duplicate output: {}", e)))?;

        if output.status.success() {
            let lb_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            self.active_loopbacks.lock().unwrap().push(lb_id.clone());
            log::info!(
                "Output duplication started from {} to {} (ID: {})",
                source_name,
                target_name,
                lb_id
            );
            Ok(())
        } else {
            Err(Error::PlatformError(format!(
                "Failed to start duplication: {}",
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }

    fn stop_all_duplications(&self) -> Result<(), Error> {
        let mut loopbacks = self.active_loopbacks.lock().unwrap();
        for lb_id in loopbacks.drain(..) {
            let _ = Command::new("pactl")
                .args(["unload-module", &lb_id])
                .status();
            log::info!("Stopped duplication module {}", lb_id);
        }
        Ok(())
    }
}

impl LinuxVirtualCable {
    /// Processes audio (wrapper for triple buffer).
    pub fn process_audio(&self, input: &[f32], output: &mut [f32]) -> Result<usize, Error> {
        if !self.is_running() {
            return Err(Error::PlatformError("Cable not running".into()));
        }
        let processed = self.triple_buffer.lock().unwrap().process(input, output)?;
        self.samples_processed
            .fetch_add(processed as u64, Ordering::Relaxed);
        Ok(processed)
    }

    fn calculate_latency(&self) -> f64 {
        let stats = self.triple_buffer.lock().unwrap().stats();
        (stats.resample_available as f64 * 1000.0) / self.config.sample_rate as f64
    }

    fn estimate_cpu_usage(&self) -> f64 {
        if self.is_running() {
            0.5
        } else {
            0.0
        }
    }
}
