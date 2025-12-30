# Virtual Audio Cable (VAC) en Rust

Una librería multiplataforma para la creación y gestión de cables de audio virtuales y enrutamiento dinámico de audio.

## 🚀 Características

- **Multiplataforma**: Soporte para Linux (PulseAudio/PipeWire) y Windows (WDM/WaveRT - *en desarrollo*).
- **Enrutamiento Dinámico**: APIs para capturar audio global del sistema o de aplicaciones específicas (ventanas).
- **Baja Latencia**: Optimizado para procesamiento de audio en tiempo real con buffers circulares lock-free.
- **Seguridad**: Desarrollado íntegramente en Rust, garantizando seguridad de memoria y concurrencia.

## 🛠 Instalación

Añade esto a tu `Cargo.toml`:

```toml
[dependencies]
virtual-audio-cable = { path = "path/to/virtual-audio-cable" }
```

## 💻 Uso Básico

### Crear un Micrófono Virtual (Global)

```rust
use virtual_audio_cable::{VirtualCable, VirtualCableTrait, CableConfig};

fn main() -> anyhow::Result<()> {
    // Configuración por defecto (48kHz, Estéreo)
    let config = CableConfig::default();
    let mut cable = VirtualCable::new(config)?;

    // Iniciar el cable virtual
    cable.start()?;
    
    // Enrutar todo el audio del sistema al cable virtual
    cable.route_system_audio()?;

    println!("Capturando audio global. Presiona Ctrl+C para detener.");
    // ... mantener vivo el proceso ...
    
    Ok(())
}
```

### Capturar Audio de una Aplicación Específica

```rust
use virtual_audio_cable::{VirtualCable, VirtualCableTrait, CableConfig};

fn main() -> anyhow::Result<()> {
    let mut cable = VirtualCable::new(CableConfig::default())?;
    cable.start()?;

    // Listar aplicaciones que están reproduciendo sonido
    let apps = cable.list_applications()?;
    for app in apps {
        if app.name.contains("Spotify") {
            println!("Enrutando Spotify (ID: {})", app.id);
            cable.route_application(&app.id)?;
        }
    }

    Ok(())
}
```

## 🧪 Tests

Para ejecutar los tests de la librería:

```bash
cargo test
```

## 📂 Ejemplos

La librería incluye varios ejemplos listos para usar:

- `virtual_microphone`: Crea un micrófono virtual básico.
- `route_app`: Demuestra cómo encontrar y enrutar el audio de una aplicación específica.
- `list_devices`: Lista los dispositivos de audio disponibles en el sistema.

Ejecútalos con:
```bash
cargo run --example nombre_del_ejemplo
```

## 🐧 Soporte Linux

En Linux, la librería utiliza `pactl` para interactuar con PulseAudio o PipeWire. Esto permite:
- Crear Null Sinks dinámicos.
- Mover flujos de audio entre dispositivos sin reiniciar las aplicaciones.
- Latencia configurable.

## 🪟 Soporte Windows

El soporte para Windows está basado en el modelo de controladores WDM/WaveRT. Actualmente es un esqueleto funcional que requiere el uso del WDK para su compilación final como controlador de kernel.

## 📄 Licencia

Este proyecto está bajo la licencia MIT o Apache-2.0.