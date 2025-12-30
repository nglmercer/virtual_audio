//! Ejemplo de API para micrófono virtual
//!
//! Este ejemplo demuestra cómo crear un micrófono virtual usando la librería
//! de Virtual Audio Cable. El micrófono virtual puede capturar audio de la salida
//! del sistema (speakers) y redirigirlo como entrada a otras aplicaciones.

use anyhow::Result;
use log::info;
use std::time::Duration;
use virtual_audio_cable::{AudioFormat, CableConfig, VirtualCable, VirtualCableTrait};

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar el logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🎤 Iniciando Micrófono Virtual");
    info!("Este ejemplo crea un dispositivo de entrada virtual que captura");
    info!("audio de la salida del sistema y lo redirige a otras aplicaciones.");
    info!("");

    // Configuración del micrófono virtual
    let config = CableConfig {
        sample_rate: 48000,       // 48 kHz (estándar para audio de alta calidad)
        channels: 2,             // Estéreo
        buffer_size: 1024,       // Tamaño del buffer (ajustar según latencia deseada)
        format: AudioFormat::F32LE, // Formato de punto flotante de 32 bits
        device_name: "Micrófono Virtual".to_string(),
    };

    info!("⚙️  Configuración del Micrófono:");
    info!("   Nombre del dispositivo: {}", config.device_name);
    info!("   Tasa de muestreo: {} Hz", config.sample_rate);
    info!("   Canales: {} (estéreo)", config.channels);
    info!("   Tamaño de buffer: {} muestras", config.buffer_size);
    info!("   Formato: {}", config.format.name());
    info!("");

    // Crear el cable virtual (micrófono)
    let mut cable = VirtualCable::new(config.clone())?;
    info!("✅ Cable virtual creado correctamente");

    // Iniciar el micrófono virtual
    cable.start()?;
    info!("🚀 Micrófono virtual iniciado correctamente");
    info!("");

    info!("📝 Instrucciones de uso:");
    info!("   1. Abre tu aplicación de grabación o conferencia (Zoom, Teams, OBS, etc.)");
    info!("   2. En la configuración de audio, selecciona '{}' como micrófono", config.device_name);
    info!("   3. El audio de tu sistema ahora será capturado por el micrófono virtual");
    info!("   4. Presiona Ctrl+C para detener");
    info!("");

    // Ejemplo de monitoreo de estadísticas
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    interval.tick().await; // Skip first tick

    loop {
        interval.tick().await;
        let stats = cable.get_stats();

        info!("📊 Estadísticas del Micrófono:");
        info!("   Estado: {}", if stats.is_running { "✓ Activo" } else { "✗ Inactivo" });
        info!("   Muestras procesadas: {}", stats.samples_processed);
        info!("   Underruns (buffer vacío): {}", stats.underruns);
        info!("   Overruns (buffer lleno): {}", stats.overruns);
        info!("   Latencia actual: {:.2} ms", stats.latency_ms);
        info!("   Uso de CPU: {:.1}%", stats.cpu_usage);
        info!("");
    }

    // El siguiente código se ejecutará al recibir Ctrl+C
    // Nota: En una aplicación real, deberías agregar un handler para la señal
    // cable.stop()?;
    // info!("🛑 Micrófono virtual detenido");
    // Ok(())
}
