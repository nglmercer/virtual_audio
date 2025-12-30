# Especificaciones Técnicas: Cable de Audio Virtual en Rust

## Estado del Sistema

### Verificación de Rust Toolchain
- **Rust Compiler**: ✅ rustc 1.92.0 (ded5c06cf 2025-12-08)
- **Cargo Package Manager**: ✅ cargo 1.92.0 (344c4567c 2025-10-21)
- **Requisito mínimo**: Rust 1.70+ recomendado para desarrollo de drivers

**Estado Actual**: ✅ Rust instalado y configurado correctamente.

---

## 1. Visión General del Proyecto

### Objetivo
Desarrollar un cable de audio virtual multiplataforma en Rust que permita:
- Conectar la salida de audio de una aplicación con la entrada de otra
- Transferencia sincrónica de datos PCM (Pulse Code Modulation)
- Baja latencia y alta estabilidad
- Compatibilidad con Windows y Linux

### Plataformas Soportadas

| Plataforma | Arquitectura | Modelo de Integración | Latencia Objetivo |
|------------|--------------|----------------------|-------------------|
| Windows 10/11 | Kernel Mode (WDM/WaveRT) | PortCls + Kernel Streaming | < 10ms (ASIO) |
| Linux (PipeWire) | User Space | Nodos de grafo SPA | < 5ms nativo |

---

## 2. Requisitos del Sistema

### Requisitos de Desarrollo

#### Windows
- **Windows 10/11** (Build 19041+)
- **Windows Driver Kit (WDK)** - Última versión
- **Windows SDK** - Compatible con WDK
- **Visual Studio 2022** (Build Tools mínimo)
- **Rust nightly** para desarrollo de drivers
- **cargo-wdk** - Extensión de Cargo para WDK
- **Certificado de prueba** para firma de drivers

#### Linux
- **Distribución Linux moderna** (Ubuntu 22.04+, Fedora 38+, Arch Linux)
- **PipeWire** >= 0.3.60
- **libpipewire-dev**
- **ALSA Development Libraries**
- **Rust stable** (1.70+)

### Requisitos de Hardware
- CPU: Procesador moderno de 64 bits (x86_64)
- RAM: 8GB mínimo, 16GB recomendado
- Almacenamiento: 5GB libres para toolchain y dependencias

---

## 3. Stack Tecnológico

### 3.1 Windows: Desarrollo de Drivers en Kernel

#### Crates Esenciales

```toml
[dependencies]
wdk = "0.3"
wdk-sys = "0.3"
wdk-alloc = "0.3"
wdk-macros = "0.3"
windows = { version = "0.58", features = ["Win32_Media_Audio"] }
```

#### Configuración del Proyecto

```toml
[package]
name = "virtual_audio"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
lto = true
opt-level = "z"
strip = true
```

#### Arquitectura del Driver

```
┌─────────────────────────────────────────┐
│      User Space Applications            │
│  (DAWs, Reproductores, etc.)           │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│      Windows Audio Engine (WASAPI)      │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│         PortCls.sys (Class Driver)      │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│    Rust Miniport Driver (WaveRT)       │
│  - Buffer Management                   │
│  - DMA Operations                      │
│  - COM Interfaces                      │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│      Virtual Audio Ring Buffer          │
│  - Input Buffer (Speaker Pin)          │
│  - Resampling Buffer                   │
│  - Output Buffer (Mic Pin)             │
└─────────────────────────────────────────┘
```

### 3.2 Linux: PipeWire User Space

#### Crates Esenciales

```toml
[dependencies]
pipewire = "0.8"
ashpd = "0.9"
tokio = { version = "1.35", features = ["full"] }
cpal = "0.15"
```

#### Arquitectura en PipeWire

```
┌─────────────────────────────────────────┐
│      User Space Applications            │
│  (Firefox, OBS, Discord, etc.)          │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│         PipeWire Audio Daemon           │
│  - Graph Management                     │
│  - Node Discovery                       │
│  - Buffer Allocation                    │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│    Rust Virtual Cable Node              │
│  - Stream Capture (sink input)          │
│  - Resampling Engine (rubato)          │
│  - Stream Playback (source output)      │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│      SPA Ring Buffer                    │
│  - Lock-free data transfer              │
│  - Atomic synchronization               │
└─────────────────────────────────────────┘
```

---

## 4. Componentes Técnicos

### 4.1 Gestión de Búferes

#### Triple Ring Buffer Architecture

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct TripleRingBuffer {
    // Buffer de entrada: recibe datos del dispositivo de captura
    ring_input: Vec<f32>,
    input_write_pos: AtomicUsize,
    input_read_pos: AtomicUsize,
    
    // Buffer de remuestreo: transforma entre sample rates
    ring_resampler: Vec<f32>,
    resample_write_pos: AtomicUsize,
    resample_read_pos: AtomicUsize,
    
    // Buffer de salida: entrega datos al dispositivo de reproducción
    ring_output: Vec<f32>,
    output_write_pos: AtomicUsize,
    output_read_pos: AtomicUsize,
}
```

**Características:**
- Tamaño de buffer configurable: 256-4096 samples
- Lock-free operations para evitar dropouts
- Soporte para planar e interleaved formats
- Sample rates: 44.1kHz, 48kHz, 96kHz

### 4.2 Formatos de Audio Soportados

| Formato | Bytes por Sample | Endianness | Uso Típico |
|---------|------------------|------------|------------|
| F32LE | 4 | Little Endian | PipeWire interno, DAWs |
| S16LE | 2 | Little Endian | VoIP, hardware legacy |
| S24LE | 3 | Little Endian | Audio profesional |
| S32LE | 4 | Little Endian | Procesamiento DSP |

### 4.3 Remuestreo en Tiempo Real

```toml
[dependencies]
rubato = "0.14"  # Resampling library
```

**Capacidades:**
- Conversiones arbitrarias (ej: 44.1k ↔ 48k)
- Quality modes: Fast, Balanced, High
- Interpolation: Linear, Cubic, Sinc
- Latencia predictiva y configurable

---

## 5. Implementación por Plataforma

### 5.1 Windows Implementation

#### 5.1.1 WaveRT Driver Structure

```rust
use wdk::*;

#[no_mangle]
pub extern "system" fn DriverEntry(
    driver_object: PDRIVER_OBJECT,
    registry_path: PUNICODE_STRING,
) -> NTSTATUS {
    // Inicialización del driver
    NTSTATUS::STATUS_SUCCESS
}

pub struct VirtualAudioMiniport {
    device_properties: KSDEVICE_DESCRIPTOR,
    filter_descriptor: KSFILTER_DESCRIPTOR,
    // ... campos específicos del dispositivo
}

impl IMiniportWaveRT for VirtualAudioMiniport {
    fn get_description(&self) -> &KSFILTER_DESCRIPTOR {
        &self.filter_descriptor
    }
    
    // Implementación de métodos requeridos
}
```

#### 5.1.2 INF Configuration

```ini
[Version]
Signature="$WINDOWS NT$"
Class=Media
ClassGuid={4d36e96c-e325-11ce-bfc1-08002be10318}
Provider=%ManufacturerName%
DriverVer=01/01/2025,1.0.0.0
CatalogFile=virtual_audio.cat

[Manufacturer]
%ManufacturerName%=Standard,NTamd64

[Standard.NTamd64]
%DeviceDescription%=DriversInstall, ROOT\VirtualAudio

[DriversInstall]
Include=ks.inf, wdmaudio.inf
Needs=KS.Registration, WDMAUDIO.Registration
CopyFiles=Drivers.CopyDrivers

[Drivers.CopyDrivers]
virtual_audio.sys,,,2

[Drivers.NT.Services]
AddService = virtual_audio, 0x00000002, Drivers.ServiceInstall

[Drivers.ServiceInstall]
DisplayName    = %ServiceDisplayName%
ServiceType    = 1               ; SERVICE_KERNEL_DRIVER
StartType      = 3               ; SERVICE_DEMAND_START
ErrorControl   = 1               ; SERVICE_ERROR_NORMAL
ServiceBinary  = %10%\System32\drivers\virtual_audio.sys

[Strings]
ManufacturerName="Your Company"
DeviceDescription="Virtual Audio Cable"
ServiceDisplayName="Virtual Audio Cable Driver"
```

#### 5.1.3 Build System

```bash
# Instalar cargo-wdk
cargo install cargo-wdk

# Inicializar proyecto de driver
cargo wdk init --name virtual_audio

# Compilar
cargo wdk build

# Empaquetar driver
cargo wdk package
```

### 5.2 Linux Implementation

#### 5.2.1 PipeWire Node Creation

```rust
use pipewire as pw;
use pipewire::prelude::*;

async fn create_virtual_cable() -> Result<(), Box<dyn std::error::Error>> {
    let mainloop = pw::MainLoop::new()?;
    let context = pw::Context::new(&mainloop)?;
    let core = context.connect()?;
    
    // Crear sink virtual (dispositivo de entrada de audio)
    let sink = pw::stream::Stream::new(
        &core,
        "virtual-cable-sink",
        &pw::properties::properties! {
            *pw::keys::MEDIA_TYPE => "Audio",
            *pw::keys::MEDIA_CATEGORY => "Playback",
            *pw::keys::MEDIA_ROLE => "Music",
            *pw::keys::NODE_NAME => "virtual_audio_sink",
        }
    )?;
    
    // Crear source virtual (dispositivo de salida/micrófono)
    let source = pw::stream::Stream::new(
        &core,
        "virtual-cable-source",
        &pw::properties::properties! {
            *pw::keys::MEDIA_TYPE => "Audio",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Music",
            *pw::keys::NODE_NAME => "virtual_audio_source",
        }
    )?;
    
    // Configurar callbacks y buffers
    sink.connect(
        pw::spa::audio::AudioFormat::F32_LE,
        &pw::stream::StreamFlags::AUTOCONNECT,
    )?;
    
    source.connect(
        pw::spa::audio::AudioFormat::F32_LE,
        &pw::stream::StreamFlags::AUTOCONNECT,
    )?;
    
    mainloop.run();
    
    Ok(())
}
```

#### 5.2.2 Audio Processing Pipeline

```rust
struct AudioProcessor {
    input_buffer: Vec<f32>,
    resampler: Resampler<f32>,
    output_buffer: Vec<f32>,
}

impl AudioProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ResampleError> {
        // 1. Escribir en buffer de entrada
        self.input_buffer.extend_from_slice(input);
        
        // 2. Resamplear si necesario
        let resampled = self.resampler.process(&self.input_buffer)?;
        
        // 3. Escribir en buffer de salida
        output.copy_from_slice(&resampled[..output.len()]);
        
        Ok(())
    }
}
```

---

## 6. Consideraciones de Seguridad y Certificación

### 6.1 Windows Driver Certification (2025)

#### Cronología Crítica

| Fecha | Evento | Impacto |
|-------|--------|---------|
| Junio 2025 | Nueva CA de pre-producción | Requiere nuevos certificados para testing |
| Julio 2025 | Expiración de CAs antiguas | Drivers firmados con certificados viejos serán rechazados |
| Septiembre 2025 | RustConf 2025 | Anuncios de Microsoft sobre abstracciones seguras |
| Diciembre 2025 | CodeQL estable para Rust | Facilita certificación WHCP |

#### Requisitos de Certificación

1. **HLK Testing** (Hardware Lab Kit)
   - Audio Driver Tests
   - System Stress Tests
   - Driver Verifier compliance

2. **Firma de Código**
   - EV Code Signing Certificate
   - Cross-signing certificate (para test)
   - Time-stamping server

3. **Compliance**
   - WHCP (Windows Hardware Compatibility Program)
   - Security compliance checks
   - Anti-tampering measures

### 6.2 Seguridad de Memoria en Rust

#### Safe vs Unsafe

```rust
// ✅ SAFE: Rust garantiza memoria correcta
fn safe_copy(src: &[f32], dst: &mut [f32]) {
    dst.copy_from_slice(src);
}

// ⚠️ UNSAFE: Requiere auditoría manual
unsafe fn unsafe_dma_copy(src: *const u8, dst: *mut u8, len: usize) {
    std::ptr::copy_nonoverlapping(src, dst, len);
}
```

**Mejores Prácticas:**
- Minimizar bloques `unsafe`
- Encapsular FFI en wrappers seguros
- Auditoría de código con Clippy y MIRI
- Pruebas exhaustivas de concurrencia

### 6.3 Real-Time Safety

#### Reglas para Callbacks de Audio

```rust
// ❌ PROHIBIDO en callbacks de audio real-time
// - Asignaciones de memoria (Vec::push, Box::new)
// - I/O blocking (filesystem, network)
// - Locks sin prioridad inherente (std::sync::Mutex)

// ✅ PERMITIDO
// - Operaciones atómicas
// - Arrays pre-alojados
// - Operaciones matemáticas puras
// - Copy en stack
```

**Crates Recomendadas:**
```toml
heapless = "0.8"  # Data structures sin alloc
atomic_float = "0.1"  # Float atómicos
crossbeam = "0.8"  # Lock-free data structures
```

---

## 7. Plan de Desarrollo

### Fase 1: Setup y Configuración (Semanas 1-2)

- [ ] Instalar Rust toolchain (stable + nightly)
- [ ] Configurar entorno de desarrollo para Windows/Linux
- [ ] Instalar WDK (Windows) o PipeWire dev libs (Linux)
- [ ] Crear estructura del proyecto
- [ ] Configurar sistema de build

### Fase 2: Core del Sistema (Semanas 3-6)

- [ ] Implementar triple ring buffer
- [ ] Desarrollar motor de remuestreo
- [ ] Crear API de gestión de formatos
- [ ] Implementar sincronización lock-free
- [ ] Unit tests de componentes

### Fase 3: Implementación Windows (Semanas 7-12)

- [ ] Crear driver skeleton con cargo-wdk
- [ ] Implementar IMiniportWaveRT
- [ ] Configurar DMA buffers
- [ ] Implementar loopback interno
- [ ] Desarrollar INF de instalación
- [ ] Testing en Windows 10/11

### Fase 4: Implementación Linux (Semanas 13-16)

- [ ] Integrar con PipeWire
- [ ] Crear nodos virtuales (sink/source)
- [ ] Implementar procesamiento de audio
- [ ] Desarrollar CLI de control
- [ ] Testing en múltiples distribuciones

### Fase 5: Optimización y Testing (Semanas 17-20)

- [ ] Performance profiling
- [ ] Latency optimization
- [ ] Stress testing
- [ ] Memory leak detection
- [ ] Cross-platform compatibility tests

### Fase 6: Documentación y Release (Semanas 21-22)

- [ ] Documentación de API
- [ ] Guías de instalación
- [ ] Ejemplos de uso
- [ ] Preparar release v1.0.0
- [ ] Publicación en crates.io

---

## 8. Métricas de Éxito

### Performance

| Métrica | Objetivo | Método de Medición |
|---------|----------|-------------------|
| Latencia (Windows) | < 10ms | WASAPI latency test |
| Latencia (Linux) | < 5ms | PipeWire latency monitor |
| CPU Usage | < 5% @ 48kHz | CPU profiling |
| Memory footprint | < 50MB | Memory usage monitor |
| Buffer underruns | 0% | Statistics logging |

### Calidad

| Métrica | Objetivo |
|---------|----------|
| Code coverage | > 90% |
| Clippy warnings | 0 |
| Unsafe blocks | < 5% del total |
| Documentation coverage | 100% de API pública |

---

## 9. Dependencias Externas

### Windows

- **Microsoft WDK**: https://docs.microsoft.com/en-us/windows-hardware/drivers/download-the-wdk
- **Windows SDK**: Incluido con Visual Studio
- **cargo-wdk**: `cargo install cargo-wdk`

### Linux

- **PipeWire**: https://pipewire.org/
- **libpipewire-dev**: `sudo apt install libpipewire-dev` (Ubuntu/Debian)
- **pipewire-rs**: https://github.com/RustAudio/pipewire-rs

### Cross-Platform

- **cpal**: https://github.com/RustAudio/cpal
- **rubato**: https://github.com/HEnquist/rubato
- **audio**: https://github.com/RustAudio/audio

---

## 10. Recursos y Referencias

### Documentación Oficial

1. **Windows Driver Development**
   - https://docs.microsoft.com/en-us/windows-hardware/drivers/
   - https://github.com/microsoft/windows-drivers-rs
   - WaveRT Porting Guide

2. **PipeWire**
   - https://docs.pipewire.org/
   - https://gitlab.freedesktop.org/pipewire/pipewire
   - PipeWire SPA Documentation

3. **Rust Audio Ecosystem**
   - https://github.com/RustAudio/rust-audio
   - https://rust-lang.github.io/async-book/

### Libros y Guías

1. "Windows Kernel Programming" - Pavel Yosifovich
2. "The Rust Programming Language" - Klabnik & Nichols
3. "Real-Time Audio Programming" - Will Pirkle

### Comunidades

- **r/rust** - https://reddit.com/r/rust
- **Rust Audio Discord** - https://discord.gg/YCpc8T8
- **Windows Driver Development Forum** - https://community.osr.com/

---

## 11. Consideraciones Futuras

### V2.0 Roadmap

- [ ] Soporte para ASIO en Windows
- [ ] Integración con JACK en Linux
- [ ] Audio effects en línea (EQ, reverb, compression)
- [ ] WebRTC integration para streaming
- [ ] Soporte para dispositivos Bluetooth
- [ ] API de scripting (Lua/Python)

### Investigación

- [ ] FPGA acceleration para DSP
- [ ] Neural audio processing
- [ ] Distributed audio streaming (ROC)
- [ ] Ambisonics y spatial audio

---

## 12. Licencia y Contribución

### Licencia
Este proyecto se distribuirá bajo una licencia dual:
- **MIT License** - Para uso comercial
- **Apache 2.0** - Para contribuciones de la comunidad

### Políticas de Contribución
- Código debe pasar Clippy (`-D warnings`)
- Todas las PR requieren aprobación de 2 maintainers
- Tests obligatorios para cambios
- Documentación requerida para API pública

---

## 13. Glosario

| Término | Definición |
|---------|------------|
| **DMA** | Direct Memory Access - Transferencia de datos sin intervención de CPU |
| **PCM** | Pulse Code Modulation - Representación digital de audio |
| **Ring Buffer** | Búfer circular para transferencia continua de datos |
| **SPA** | Simple Plugin API - Sistema de plugins de PipeWire |
| **WaveRT** | Wave Real-Time - Modelo de driver de baja latencia en Windows |
| **BSoD** | Blue Screen of Death - Fallo crítico de Windows en kernel mode |
| **ASIO** | Audio Stream Input/Output - Protocolo de baja latencia en Windows |
| **WASAPI** | Windows Audio Session API - API de audio moderna de Windows |
| **KSCATEGORY** | KS Device Categories - Categorías de dispositivos en kernel streaming |

---

**Versión del Documento**: 1.0.0  
**Fecha**: 30 de Diciembre, 2025  
**Autor**: Virtual Audio Cable Team  
**Status**: Draft - Pendiente de Revisión
