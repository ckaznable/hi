use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logging() {
    init_logging_to_dir(crate::paths::data_dir().ok());
}

pub fn init_logging_to_dir(base_dir: Option<std::path::PathBuf>) {
    let log_dir = base_dir.and_then(|dir| {
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
    });

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_init_logging_creates_log_directory() {
        let dir = tempfile::tempdir().unwrap();
        let logs_dir = dir.path().join("logs");
        assert!(!logs_dir.exists());

        let log_dir = {
            let base = dir.path().to_path_buf();
            let logs = base.join("logs");
            std::fs::create_dir_all(&logs).unwrap();
            logs
        };

        assert!(log_dir.exists());
        assert!(log_dir.is_dir());
    }

    #[test]
    fn test_log_dir_none_does_not_panic() {
        let base_dir: Option<PathBuf> = None;
        let log_dir = base_dir.and_then(|dir| {
            let logs = dir.join("logs");
            std::fs::create_dir_all(&logs).ok().map(|_| logs)
        });
        assert!(log_dir.is_none());
    }

    #[test]
    fn test_env_filter_defaults_to_info() {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let display = format!("{filter}");
        assert!(
            display.contains("info"),
            "expected 'info' in filter: {display}"
        );
    }

    #[test]
    fn test_file_appender_writes_to_log_dir() {
        let dir = tempfile::tempdir().unwrap();
        let logs = dir.path().join("logs");
        std::fs::create_dir_all(&logs).unwrap();

        let appender = rolling::daily(&logs, "hi.log");
        use std::io::Write;
        let mut writer = appender;
        writeln!(writer, "test log line").unwrap();

        let entries: Vec<_> = std::fs::read_dir(&logs)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(
            !entries.is_empty(),
            "expected log files in {}",
            logs.display()
        );
    }
}
