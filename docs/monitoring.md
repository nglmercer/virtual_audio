# Statistics & Monitoring

Complete guide to monitoring virtual audio cable performance and health.

## CableStats Structure

```rust
pub struct CableStats {
    pub is_running: bool,          // Cable operational status
    pub samples_processed: u64,    // Total samples processed
    pub underruns: u64,          // Buffer underflow events
    pub overruns: u64,           // Buffer overflow events
    pub latency_ms: f64,         // Current latency in milliseconds
    pub cpu_usage: f64,          // CPU usage percentage (0.0-100.0)
}
```

## Getting Statistics

Retrieve current statistics at any time:

```rust
use virtual_audio_cable::VirtualCableTrait;

let stats = cable.get_stats();

println!("Status: {}", if stats.is_running { "Running" } else { "Stopped" });
println!("Samples: {}", stats.samples_processed);
println!("Latency: {:.2}ms", stats.latency_ms);
println!("CPU: {:.1}%", stats.cpu_usage);
println!("Underruns: {}", stats.underruns);
println!("Overruns: {}", stats.overruns);
```

## Continuous Monitoring

### Basic Periodic Monitoring

```rust
use std::time::Duration;

tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let stats = cable.get_stats();
        println!("Latency: {:.2}ms, CPU: {:.1}%", 
                 stats.latency_ms, stats.cpu_usage);
    }
});
```

### Advanced Monitoring with Alerts

```rust
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    
    loop {
        interval.tick().await;
        let stats = cable.lock().unwrap().get_stats();

        // Check for issues
        if stats.underruns > 10 {
            log::warn!("High underrun count: {}", stats.underruns);
        }
        
        if stats.overruns > 10 {
            log::warn!("High overrun count: {}", stats.overruns);
        }
        
        if stats.latency_ms > 50.0 {
            log::warn!("High latency: {:.2}ms", stats.latency_ms);
        }
        
        if stats.cpu_usage > 20.0 {
            log::warn!("High CPU usage: {:.1}%", stats.cpu_usage);
        }

        // Log status
        log::info!("Running: {}, Samples: {}, Latency: {:.2}ms, CPU: {:.1}%",
                  stats.is_running,
                  stats.samples_processed,
                  stats.latency_ms,
                  stats.cpu_usage);
    }
});
```

## Understanding Metrics

### is_running

**What it means:** Whether the cable is actively processing audio.

**Interpretation:**
- `true` - Cable is operational
- `false` - Cable is stopped or failed

**Use case:**
```rust
if !stats.is_running {
    log::error!("Cable is not running!");
    // Attempt restart or alert user
}
```

### samples_processed

**What it means:** Total number of audio samples processed since start.

**Interpretation:**
- Increases continuously while running
- Rate depends on sample rate and channels

**Calculate audio duration:**
```rust
let bytes_per_second = config.sample_rate * config.channels * config.format.bytes_per_sample();
let total_bytes = stats.samples_processed * config.format.bytes_per_sample() as u64;
let duration_seconds = total_bytes as f64 / bytes_per_second as f64;

println!("Audio processed: {:.2} minutes", duration_seconds / 60.0);
```

### underruns

**What it means:** Number of times the output buffer was empty when audio was requested.

**Causes:**
- Buffer size too small
- CPU cannot keep up
- System under heavy load

**Solutions:**
1. Increase `buffer_size`
2. Reduce `sample_rate`
3. Use mono instead of stereo
4. Upgrade hardware

```rust
if stats.underruns > 100 {
    log::warn!("Too many underruns! Consider increasing buffer size.");
}
```

### overruns

**What it means:** Number of times the input buffer was full when audio arrived.

**Causes:**
- Buffer size too small
- Consumer cannot keep up
- Audio source too fast

**Solutions:**
1. Increase `buffer_size`
2. Optimize consumer code
3. Check system performance

```rust
if stats.overruns > 100 {
    log::warn!("Too many overruns! Consider increasing buffer size.");
}
```

### latency_ms

**What it means:** Current audio delay through the cable.

**Interpretation:**
- Lower is better for real-time applications
- Higher is more stable
- Target: < 50ms for most applications

**Latency guidelines:**
| Use Case | Target Latency |
|-----------|----------------|
| Gaming     | < 20ms         |
| VoIP       | < 50ms         |
| Music      | < 100ms        |
| Recording  | < 200ms        |

**Calculate expected latency:**
```rust
let expected_latency = (config.buffer_size as f64 * 1000.0) / config.sample_rate as f64;
println!("Expected latency: {:.2}ms", expected_latency);
```

### cpu_usage

**What it means:** CPU resources used by the cable (0.0-100.0%).

**Interpretation:**
- < 5% - Excellent
- 5-10% - Good
- 10-20% - Acceptable
- > 20% - High, consider optimization

**CPU optimization tips:**
```rust
if stats.cpu_usage > 20.0 {
    log::warn!("High CPU usage! Try:");
    log::warn!("  1. Increase buffer_size");
    log::warn!("  2. Decrease sample_rate");
    log::warn!("  3. Use S16LE format instead of F32LE");
    log::warn!("  4. Use mono instead of stereo");
}
```

## Performance Dashboards

### Simple Console Dashboard

```rust
fn print_dashboard(stats: &CableStats) {
    print!("\x1B[2J\x1B[H"); // Clear screen
    println!("┌─────────────────────────────────────┐");
    println!("│     VIRTUAL AUDIO CABLE MONITOR    │");
    println!("├─────────────────────────────────────┤");
    println!("│ Status: {}{}│",
              if stats.is_running { "✓ Running" } else { "✗ Stopped" },
              " ".repeat(12));
    println!("│                                  │");
    println!("│ Samples:  {:>20}      │", stats.samples_processed);
    println!("│ Latency:  {:>18.2}ms  │", stats.latency_ms);
    println!("│ CPU:      {:>20.1}% │", stats.cpu_usage);
    println!("│                                  │");
    println!("│ Underruns: {:>17}      │", stats.underruns);
    println!("│ Overruns:  {:>18}      │", stats.overruns);
    println!("└─────────────────────────────────────┘");
}
```

### Web Dashboard (with Actix)

```rust
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Serialize;

#[derive(Serialize)]
struct StatsResponse {
    is_running: bool,
    samples_processed: u64,
    latency_ms: f64,
    cpu_usage: f64,
}

async fn get_stats(
    cable: web::Data<Arc<Mutex<VirtualCable>>>
) -> HttpResponse {
    let stats = cable.lock().unwrap().get_stats();
    let response = StatsResponse {
        is_running: stats.is_running,
        samples_processed: stats.samples_processed,
        latency_ms: stats.latency_ms,
        cpu_usage: stats.cpu_usage,
    };
    HttpResponse::Ok().json(response)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cable = Arc::new(Mutex::new(VirtualCable::new(
        CableConfig::default()
    ).unwrap()));
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&cable)))
            .route("/stats", web::get().to(get_stats))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Logging Statistics

### Structured Logging

```rust
use log::{info, warn, error};

tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        let stats = cable.lock().unwrap().get_stats();
        
        info!(
            "cable_stats: running={}, samples={}, latency_ms={:.2}, cpu_percent={:.1}, underruns={}, overruns={}",
            stats.is_running,
            stats.samples_processed,
            stats.latency_ms,
            stats.cpu_usage,
            stats.underruns,
            stats.overruns
        );
    }
});
```

### File Logging

```rust
use std::fs::{File, OpenOptions};
use std::io::Write;

let mut log_file = OpenOptions::new()
    .create(true)
    .append(true)
    .open("cable_stats.csv")?;

// Write header
writeln!(log_file, "timestamp,samples,latency_ms,cpu,underruns,overruns")?;

tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let stats = cable.lock().unwrap().get_stats();
        
        writeln!(
            log_file,
            "{},{},{:.2},{:.1},{},{}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            stats.samples_processed,
            stats.latency_ms,
            stats.cpu_usage,
            stats.underruns,
            stats.overruns
        ).unwrap();
    }
});
```

## Alerting

### Email Alerts

```rust
use lettre::{Message, SmtpTransport, Transport};

async fn send_alert(subject: &str, body: &str) {
    let email = Message::builder()
        .from("alerts@example.com")
        .to("admin@example.com")
        .subject(subject)
        .body(body.to_string())
        .unwrap();
    
    let mailer = SmtpTransport::relay("smtp.example.com");
    match mailer.send(&email) {
        Ok(_) => log::info!("Alert sent successfully"),
        Err(e) => log::error!("Failed to send alert: {}", e),
    }
}

// In monitoring loop
if stats.underruns > 1000 {
    send_alert(
        "High Underrun Count",
        &format!("Underruns exceeded 1000: {}", stats.underruns)
    ).await;
}
```

### Slack Webhook

```rust
async fn send_slack_alert(message: &str) {
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "text": message
    });
    
    match client
        .post("https://hooks.slack.com/services/YOUR/WEBHOOK/URL")
        .json(&payload)
        .send()
        .await
    {
        Ok(_) => log::info!("Slack alert sent"),
        Err(e) => log::error!("Failed to send Slack alert: {}", e),
    }
}

// In monitoring loop
if stats.latency_ms > 100.0 {
    send_slack_alert(
        &format!("⚠ High latency detected: {:.2}ms", stats.latency_ms)
    ).await;
}
```

## Health Checks

```rust
async fn health_check(cable: &Arc<Mutex<VirtualCable>>) -> bool {
    let stats = cable.lock().unwrap().get_stats();
    
    // Check if running
    if !stats.is_running {
        log::error!("Health check failed: Cable not running");
        return false;
    }
    
    // Check latency
    if stats.latency_ms > 100.0 {
        log::error!("Health check failed: High latency");
        return false;
    }
    
    // Check CPU
    if stats.cpu_usage > 50.0 {
        log::error!("Health check failed: High CPU usage");
        return false;
    }
    
    // Check errors
    if stats.underruns > 100 || stats.overruns > 100 {
        log::error!("Health check failed: Too many buffer errors");
        return false;
    }
    
    true
}

// Run periodic health checks
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        if !health_check(&cable).await {
            log::warn!("Health check failed, attempting recovery...");
            // Recovery logic
        }
    }
});
```

## Best Practices

1. **Monitor regularly but not too frequently**
   ```rust
   // Good: Every 5-10 seconds
   tokio::time::sleep(Duration::from_secs(5)).await;
   
   // Bad: Every 10ms (too frequent)
   tokio::time::sleep(Duration::from_millis(10)).await;
   ```

2. **Alert on thresholds, not every event**
   ```rust
   // Good: Alert when threshold exceeded
   if stats.underruns > 100 {
       send_alert(...);
   }
   
   // Bad: Alert on every underrun
   if stats.underruns > 0 {
       send_alert(...);  // Too many alerts!
   }
   ```

3. **Log statistics for analysis**
   ```rust
   // Keep historical data for trend analysis
   writeln!(log_file, "{},{},{:.2}", timestamp, stats.samples_processed, stats.latency_ms)?;
   ```

4. **Use appropriate sampling rates**
   ```rust
   // Development: High frequency for debugging
   let interval = Duration::from_secs(1);
   
   // Production: Lower frequency for performance
   let interval = Duration::from_secs(30);
   ```

## Next Steps

- [API Overview](./api-overview.md) - Core API reference
- [Configuration](./configuration.md) - Adjust settings for better metrics
- [Error Handling](./errors.md) - Handle monitoring errors
- [Example Microphone](./example-microphone.md) - See monitoring in action
