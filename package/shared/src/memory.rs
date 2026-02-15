use crate::config::MemoryConfig;
use tracing::{debug, info};

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            large_release_threshold_bytes: 1_048_576,
        }
    }
}

pub fn should_reclaim(config: &MemoryConfig, released_bytes: usize) -> bool {
    released_bytes >= config.large_release_threshold_bytes
}

pub fn evaluate_reclamation(config: &MemoryConfig, released_bytes: usize) {
    if should_reclaim(config, released_bytes) {
        info!(
            released_bytes,
            threshold = config.large_release_threshold_bytes,
            action = "reclaim",
            "Memory reclamation"
        );
    } else {
        debug!(
            released_bytes,
            threshold = config.large_release_threshold_bytes,
            action = "skip",
            "Memory reclamation"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_reclaim_above_threshold() {
        let config = MemoryConfig {
            large_release_threshold_bytes: 1000,
        };
        assert!(should_reclaim(&config, 1000));
        assert!(should_reclaim(&config, 2000));
    }

    #[test]
    fn test_should_reclaim_below_threshold() {
        let config = MemoryConfig {
            large_release_threshold_bytes: 1000,
        };
        assert!(!should_reclaim(&config, 999));
        assert!(!should_reclaim(&config, 0));
    }

    #[test]
    fn test_default_config() {
        let config = MemoryConfig::default();
        assert_eq!(config.large_release_threshold_bytes, 1_048_576);
    }

    #[test]
    fn test_evaluate_reclamation_skip() {
        let config = MemoryConfig {
            large_release_threshold_bytes: 1_000_000,
        };
        evaluate_reclamation(&config, 100);
    }

    #[test]
    fn test_evaluate_reclamation_reclaim() {
        let config = MemoryConfig {
            large_release_threshold_bytes: 100,
        };
        evaluate_reclamation(&config, 200);
    }
}
