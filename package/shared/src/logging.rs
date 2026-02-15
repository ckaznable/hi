use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize the tracing logging infrastructure.
///
/// - File layer: writes warn+ to `data_dir()/logs/hi.log` with daily rotation.
/// - Stderr layer: writes info+ (or whatever RUST_LOG specifies) to stderr.
///
/// Falls back to stderr-only if the log directory cannot be created.
pub fn init_logging() {
    let log_dir = match crate::paths::data_dir() {
        Ok(dir) => {
            let logs = dir.join("logs");
            match std::fs::create_dir_all(&logs) {
                Ok(()) => Some(logs),
                Err(e) => {
                    eprintln!(
                        "[logging] Failed to create log directory: {e}. Falling back to stderr-only."
                    );
                    None
                }
            }
        }
        Err(e) => {
            eprintln!(
                "[logging] Failed to resolve data directory: {e}. Falling back to stderr-only."
            );
            None
        }
    };

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match log_dir {
        Some(dir) => {
            let file_appender = rolling::daily(&dir, "hi.log");
            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_target(true);

            let stderr_layer = fmt::layer().with_writer(std::io::stderr).with_target(true);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(file_layer)
                .with(stderr_layer)
                .init();
        }
        None => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().with_writer(std::io::stderr))
                .init();
        }
    }
}
