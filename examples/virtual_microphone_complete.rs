//! Ejemplo completo de API para micrófono virtual con manejo de señales
//!
//! Este ejemplo demuestra cómo crear un micrófono virtual con todas las
//! funcionalidades: configuración, inicio, monitoreo y detención elegante.

use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use virtual_audio_cable::{AudioFormat, CableConfig, VirtualCable, VirtualCableTrait};

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar el logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🎤 Micrófono Virtual - Ejemplo Completo");
    info!("{}", "=".repeat(60));
    info!("");

    // Configuración personalizada del micrófono virtual
    let config = CableConfig {
        sample_rate: 48000,         // Tasa de muestreo en Hz
        channels: 2,                // 1 = mono, 2 = estéreo
        buffer_size: 2048,          // Tamaño del buffer (mayor = más latencia, más estabilidad)
        format: AudioFormat::F32LE, // Formato: F32LE, S16LE, S24LE, S32LE
        device_name: "Mi Micrófono Virtual".to_string(),
    };

    info!("⚙️  Configuración:");
    info!("   Nombre: {}", config.device_name);
    info!("   Tasa de muestreo: {} Hz", config.sample_rate);
    info!(
        "   Canales: {}",
        if config.channels == 1 {
            "Mono"
        } else {
            "Estéreo"
        }
    );
    info!(
        "   Tamaño de buffer: {} muestras (~{:.1} ms de latencia)",
        config.buffer_size,
        (config.buffer_size as f64 * 1000.0 / config.sample_rate as f64)
    );
    info!("   Formato: {}", config.format.name());
    info!(
        "   Bytes por muestra: {} bytes",
        config.format.bytes_per_sample()
    );
    info!("");

    // Crear el cable virtual
    let cable = Arc::new(std::sync::Mutex::new(VirtualCable::new(config.clone())?));
    info!("✅ Cable virtual creado");
    info!("");

    // Iniciar el micrófono virtual
    cable.lock().unwrap().start()?;
    info!("🚀 Micrófono virtual iniciado");
    info!("");

    info!("📋 CÓMO USARLO:");
    info!("────────────────────────────────────────────────────────────");
    info!("");
    info!("1️⃣  En tu aplicación de videoconferencia (Zoom, Teams, etc.):");
    info!("   - Ve a Configuración → Audio → Micrófono");
    info!("   - Selecciona '{}'", config.device_name);
    info!("");
    info!("2️⃣  En tu software de grabación (OBS, Audacity, etc.):");
    info!("   - Agrega una nueva fuente de audio");
    info!("   - Selecciona '{}' como entrada", config.device_name);
    info!("");
    info!("3️⃣  El audio de tu sistema será capturado y enviado como entrada");
    info!("");
    info!("4️⃣  Para detener: Presiona Ctrl+C");
    info!("────────────────────────────────────────────────────────────");
    info!("");

    // Iniciar el monitoreo de estadísticas en un task separado
    let cable_clone = Arc::clone(&cable);
    let monitor_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        interval.tick().await; // Skip first tick

        loop {
            interval.tick().await;
            let stats = cable_clone.lock().unwrap().get_stats();

            // Mostrar estadísticas de forma compacta
            print!("\r📊 ");
            if stats.is_running {
                print!("✓ Activo | ");
            } else {
                print!("✗ Inactivo | ");
            }
            print!("Muestras: {} | ", stats.samples_processed);

            if stats.underruns > 0 || stats.overruns > 0 {
                print!(
                    "⚠ Underruns: {} Overruns: {} | ",
                    stats.underruns, stats.overruns
                );
            }

            print!(
                "Latencia: {:.1}ms | CPU: {:.1}%",
                stats.latency_ms, stats.cpu_usage
            );

            // Advertencias de rendimiento
            if stats.latency_ms > 50.0 {
                print!(" ⚠ Latencia alta!");
            }
            if stats.underruns > 10 || stats.overruns > 10 {
                print!(" ⚠ Problemas de buffer!");
            }

            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
    });

    info!("✨ Monitoreo activo. Presiona Ctrl+C para detener...");

    // Esperar señal de interrupción (Ctrl+C)
    #[cfg(unix)]
    let _shutdown_result = signal::ctrl_c().await;

    #[cfg(windows)]
    {
        let mut ctrl_break = tokio::signal::windows::ctrl_break()?;
        tokio::select! {
            _ = signal::ctrl_c() => {}
            _ = ctrl_break.recv() => {}
        }
    }

    // Detener el monitoreo
    monitor_handle.abort();

    info!("");
    info!("");
    info!("🛑 Recibida señal de apagado. Deteniendo...");

    // Obtener estadísticas finales
    let final_stats = cable.lock().unwrap().get_stats();
    info!("");
    info!("📊 ESTADÍSTICAS FINALES:");
    info!(
        "   Estado final: {}",
        if final_stats.is_running {
            "Activo"
        } else {
            "Inactivo"
        }
    );
    info!(
        "   Total de muestras procesadas: {}",
        final_stats.samples_processed
    );
    info!("   Underruns totales: {}", final_stats.underruns);
    info!("   Overruns totales: {}", final_stats.overruns);
    info!("   Latencia final: {:.2} ms", final_stats.latency_ms);
    info!("   Uso promedio de CPU: {:.1}%", final_stats.cpu_usage);
    info!("");

    // Detener el cable
    match cable.lock().unwrap().stop() {
        Ok(_) => info!("✅ Micrófono virtual detenido correctamente"),
        Err(e) => error!("❌ Error al detener: {}", e),
    }

    info!("");
    info!("👋 ¡Gracias por usar el Micrófono Virtual!");
    info!("");

    Ok(())
}

/// Función auxiliar para crear configuración con diferentes presets
#[allow(dead_code)] // Helper function for different preset configurations
fn create_preset_config(preset: &str) -> CableConfig {
    match preset {
        "high_quality" => CableConfig {
            sample_rate: 96000,
            channels: 2,
            buffer_size: 4096,
            format: AudioFormat::F32LE,
            device_name: "High Quality Mic".to_string(),
        },
        "low_latency" => CableConfig {
            sample_rate: 48000,
            channels: 1,
            buffer_size: 512,
            format: AudioFormat::S16LE,
            device_name: "Low Latency Mic".to_string(),
        },
        _ => CableConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = CableConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 2);
    }

    #[test]
    fn test_preset_configs() {
        let high_quality = create_preset_config("high_quality");
        assert_eq!(high_quality.sample_rate, 96000);

        let low_latency = create_preset_config("low_latency");
        assert_eq!(low_latency.buffer_size, 512);
    }
}
