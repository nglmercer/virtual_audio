use anyhow::Result;
use log::info;
use std::time::Duration;
use virtual_audio_cable::{CableConfig, VirtualCable, VirtualCableTrait};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🚀 Ejemplo de Duplicación de Salida");

    let mut cable = VirtualCable::new(CableConfig::default())?;
    cable.start()?;

    // 1. Listar salidas disponibles
    info!("🔍 Listando dispositivos de salida...");
    let outputs = cable.list_outputs()?;
    
    if outputs.len() < 2 {
        info!("❌ Se necesitan al menos 2 dispositivos de salida para este ejemplo.");
        info!("   (Uno físico y el cable virtual recién creado)");
    } else {
        info!("✅ Dispositivos de salida encontrados:");
        let mut physical_out = None;
        let mut virtual_out = None;

        for out in &outputs {
            info!("   - {} [{}] {}", 
                if out.is_default { "🌟" } else { "  " },
                out.name, 
                out.description
            );
            
            // Intentar identificar un dispositivo físico (no el cable virtual)
            if !out.name.contains("Virtual_Audio_Cable") && out.is_default {
                physical_out = Some(out.clone());
            }
            if out.name.contains("Virtual_Audio_Cable") {
                virtual_out = Some(out.clone());
            }
        }

        if let (Some(src), Some(dst)) = (physical_out, virtual_out) {
            info!("🎯 Duplicando audio de '{}' hacia '{}'...", src.description, dst.description);
            cable.duplicate_output(&src.name, &dst.name)?;
            
            info!("🎤 Ahora todo lo que suene en tus altavoces también se enviará al cable virtual.");
            info!("⏳ Manteniendo la duplicación por 20 segundos...");
            tokio::time::sleep(Duration::from_secs(20)).await;

            info!("↩️  Deteniendo duplicación...");
            cable.stop_all_duplications()?;
        } else {
            info!("⚠️ No se pudo identificar automáticamente el origen y destino para la duplicación.");
        }
    }

    cable.stop()?;
    info!("✅ Fin del ejemplo.");
    Ok(())
}
