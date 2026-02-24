//! Telemetry and logging initialization.
//!
//! Sets up structured logging with tracing and optional JSON output.

use std::sync::Mutex;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// Global guard storage to prevent file handle leak
// The guard must live for the entire program duration
static LOG_GUARD: Mutex<Option<tracing_appender::non_blocking::WorkerGuard>> = Mutex::new(None);

/// Initialize telemetry (logging and tracing).
pub fn init_telemetry(
    log_level: &str,
    json_format: bool,
) -> anyhow::Result<()> {
    let filter = EnvFilter::try_new(log_level)?;

    if json_format {
        // JSON format for production
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json())
            .init();
    } else {
        // Pretty format for development
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().pretty())
            .init();
    }

    Ok(())
}

/// Initialize telemetry with file output.
pub fn init_telemetry_with_file(
    log_level: &str,
    log_file: &std::path::Path,
) -> anyhow::Result<()> {
    let filter = EnvFilter::try_new(log_level)?;
    
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;

    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(non_blocking))
        .init();

    // Store guard in global static to keep it alive for program duration
    // This prevents the file handle from being closed prematurely
    if let Ok(mut g) = LOG_GUARD.lock() {
        *g = Some(guard);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_telemetry() {
        // This would panic if called twice in tests
        // Just test the function exists
        let _ = init_telemetry("info", false);
    }
}
