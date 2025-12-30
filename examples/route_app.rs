use anyhow::Result;
use log::info;
use std::time::Duration;
use virtual_audio_cable::{CableConfig, VirtualCable, VirtualCableTrait};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🚀 Ejemplo de Enrutamiento de Aplicación Específica");

    let mut cable = VirtualCable::new(CableConfig::default())?;
    cable.start()?;

    info!("🔍 Listando aplicaciones con audio activo...");
    let apps = cable.list_applications()?;

    if apps.is_empty() {
        info!("❌ No se encontraron aplicaciones reproduciendo audio.");
        info!("   Abre Spotify, YouTube o Discord y vuelve a intentarlo.");
    } else {
        info!("✅ Aplicaciones encontradas:");
        for app in &apps {
            info!("   - [ID: {}] {} (PID: {:?})", app.id, app.name, app.pid);
        }

        // Intentar enrutar la primera aplicación encontrada
        let target = &apps[0];
        info!("🎯 Enrutando '{}' al cable virtual...", target.name);
        cable.route_application(&target.id)?;

        info!(
            "🎤 El audio de '{}' ahora está en el cable virtual.",
            target.name
        );
        info!(
            "   Puedes verificarlo en la configuración de sonido o grabando el micrófono virtual."
        );

        info!("⏳ Manteniendo el enrutamiento por 15 segundos...");
        tokio::time::sleep(Duration::from_secs(15)).await;

        info!("↩️  Restaurando el audio original de '{}'...", target.name);
        cable.unroute_application(&target.id)?;
    }

    cable.stop()?;
    info!("✅ Fin del ejemplo.");
    Ok(())
}
