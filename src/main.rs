//! Virtual Audio Cable - Main Entry Point
//!
//! This is the command-line interface for the virtual audio cable application.

use anyhow::Result;
use env_logger::Env;
use log::{error, info, warn};
use std::time::Duration;
use tokio::signal;
use virtual_audio_cable::{CableConfig, VirtualCable, VirtualCableTrait};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .init();
    
    info!("Virtual Audio Cable v0.1.0");
    info!("Cross-platform virtual audio routing in Rust");
    
    // Parse command line arguments
    let args = parse_args();
    
    // Create configuration
    let config = CableConfig {
        sample_rate: args.sample_rate,
        channels: args.channels,
        buffer_size: args.buffer_size,
        format: args.format,
        device_name: args.device_name.clone(),
    };
    
    info!("Configuration:");
    info!("  Sample Rate: {} Hz", config.sample_rate);
    info!("  Channels: {}", config.channels);
    info!("  Buffer Size: {} samples", config.buffer_size);
    info!("  Format: {}", config.format.name());
    info!("  Device Name: {}", config.device_name);
    
    // Create virtual cable
    let mut cable = VirtualCable::new(config.clone())?;
    
    // Start the cable
    cable.start()?;
    info!("Virtual audio cable started successfully");
    
    // Monitor stats if requested
    if args.monitor {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                let stats = cable.get_stats();
                info!(
                    "Stats: running={}, samples={}, underruns={}, overruns={}, latency={:.2}ms, cpu={:.1}%",
                    stats.is_running,
                    stats.samples_processed,
                    stats.underruns,
                    stats.overruns,
                    stats.latency_ms,
                    stats.cpu_usage
                );
            }
        });
    }
    
    // Wait for Ctrl+C
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
        _ = signal::ctrl_break() => {
            info!("Received break signal");
        }
    }
    
    // Stop the cable
    cable.stop()?;
    info!("Virtual audio cable stopped");
    
    Ok(())
}

/// Command line arguments
struct Args {
    sample_rate: u32,
    channels: u16,
    buffer_size: usize,
    format: virtual_audio_cable::AudioFormat,
    device_name: String,
    monitor: bool,
}

/// Parse command line arguments
fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    
    let mut sample_rate = 48000;
    let mut channels = 2;
    let mut buffer_size = 1024;
    let mut format = virtual_audio_cable::AudioFormat::F32LE;
    let mut device_name = "Virtual Audio Cable".to_string();
    let mut monitor = false;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-r" | "--sample-rate" => {
                if i + 1 < args.len() {
                    sample_rate = args[i + 1].parse().expect("Invalid sample rate");
                    i += 2;
                } else {
                    error!("Missing value for {}", args[i]);
                    std::process::exit(1);
                }
            }
            "-c" | "--channels" => {
                if i + 1 < args.len() {
                    channels = args[i + 1].parse().expect("Invalid channel count");
                    i += 2;
                } else {
                    error!("Missing value for {}", args[i]);
                    std::process::exit(1);
                }
            }
            "-b" | "--buffer" => {
                if i + 1 < args.len() {
                    buffer_size = args[i + 1].parse().expect("Invalid buffer size");
                    i += 2;
                } else {
                    error!("Missing value for {}", args[i]);
                    std::process::exit(1);
                }
            }
            "-f" | "--format" => {
                if i + 1 < args.len() {
                    format = match args[i + 1].as_str() {
                        "f32" | "F32" => virtual_audio_cable::AudioFormat::F32LE,
                        "s16" | "S16" => virtual_audio_cable::AudioFormat::S16LE,
                        "s24" | "S24" => virtual_audio_cable::AudioFormat::S24LE,
                        "s32" | "S32" => virtual_audio_cable::AudioFormat::S32LE,
                        _ => {
                            error!("Invalid format: {}", args[i + 1]);
                            std::process::exit(1);
                        }
                    };
                    i += 2;
                } else {
                    error!("Missing value for {}", args[i]);
                    std::process::exit(1);
                }
            }
            "-n" | "--name" => {
                if i + 1 < args.len() {
                    device_name = args[i + 1].clone();
                    i += 2;
                } else {
                    error!("Missing value for {}", args[i]);
                    std::process::exit(1);
                }
            }
            "-m" | "--monitor" => {
                monitor = true;
                i += 1;
            }
            _ => {
                warn!("Unknown argument: {}", args[i]);
                i += 1;
            }
        }
    }
    
    Args {
        sample_rate,
        channels,
        buffer_size,
        format,
        device_name,
        monitor,
    }
}

/// Print help information
fn print_help() {
    println!("Virtual Audio Cable v0.1.0");
    println!("Cross-platform virtual audio routing in Rust");
    println!();
    println!("USAGE:");
    println!("  virtual-audio-cable [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  -r, --sample-rate RATE    Sample rate in Hz (default: 48000)");
    println!("  -c, --channels N          Number of channels (default: 2)");
    println!("  -b, --buffer SIZE         Buffer size in samples (default: 1024)");
    println!("  -f, --format FORMAT       Audio format: f32, s16, s24, s32 (default: f32)");
    println!("  -n, --name NAME          Device name (default: 'Virtual Audio Cable')");
    println!("  -m, --monitor            Monitor and print statistics");
    println!("  -h, --help               Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("  virtual-audio-cable");
    println!("  virtual-audio-cable --sample-rate 44100 --monitor");
    println!("  virtual-audio-cable -c 1 -b 2048 -f s16");
    println!();
    println!("PLATFORMS:");
    println!("  Linux: Uses PipeWire (user-space)");
    println!("  Windows: Uses WDM/WaveRT (kernel driver)");
}
