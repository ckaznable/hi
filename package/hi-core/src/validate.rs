use std::path::PathBuf;

use shared::config::ModelConfig;

use crate::provider::create_agent_from_parts;

/// Categories of validation failure, each with a user-facing label and hint.
#[derive(Debug, PartialEq)]
pub enum ValidationErrorKind {
    /// Config file missing, unreadable, or malformed.
    ConfigLoad,
    /// Provider rejected credentials (e.g. 401, invalid key).
    AuthFailure,
    /// Network/connection/DNS/TLS/timeout error reaching the endpoint.
    NetworkFailure,
    /// Provider says the requested model does not exist or is unavailable.
    ModelNotAvailable,
    /// Provider or config error that doesn't fit the above categories.
    Unknown,
}

impl std::fmt::Display for ValidationErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigLoad => write!(f, "configuration error"),
            Self::AuthFailure => write!(f, "authentication error"),
            Self::NetworkFailure => write!(f, "network/endpoint error"),
            Self::ModelNotAvailable => write!(f, "model not available"),
            Self::Unknown => write!(f, "unknown error"),
        }
    }
}

/// Structured validation failure with classification, detail, and remediation.
pub struct ValidationError {
    pub kind: ValidationErrorKind,
    pub message: String,
    pub hint: String,
    /// Provider name (if config was loaded successfully).
    pub provider: Option<String>,
    /// Model name (if config was loaded successfully).
    pub model: Option<String>,
}

/// Successful validation result.
pub struct ValidationSuccess {
    pub provider: String,
    pub model: String,
    /// Truncated snippet of the model response.
    pub response_snippet: String,
}

/// Validate the current config by loading it and sending a probe request.
///
/// This is read-only: no config files, history, or other state is mutated.
pub async fn validate_config(
    config_path: Option<PathBuf>,
) -> std::result::Result<ValidationSuccess, ValidationError> {
    let config = match config_path {
        Some(ref p) => ModelConfig::load_from_path(p),
        None => ModelConfig::load(),
    }
    .map_err(|e| ValidationError {
        kind: ValidationErrorKind::ConfigLoad,
        message: e.to_string(),
        hint:
            "Run `hi init` to create a starter config, then edit it with your provider and API key."
                .to_string(),
        provider: None,
        model: None,
    })?;

    let provider_name = config.provider.to_string();
    let model_name = config.model.clone();

    let agent = create_agent_from_parts(
        &config.provider,
        &config.model,
        &config.api_key,
        &config.api_base,
        Some("Respond briefly."),
        vec![],
    )
    .map_err(|e| classify_agent_build_error(e, &provider_name, &model_name))?;

    let response = agent
        .chat("hi", vec![])
        .await
        .map_err(|e| classify_prompt_error(e, &provider_name, &model_name))?;

    let snippet = truncate_response(&response, 80);

    Ok(ValidationSuccess {
        provider: provider_name,
        model: model_name,
        response_snippet: snippet,
    })
}

fn truncate_response(response: &str, max_len: usize) -> String {
    let trimmed = response.trim();
    if trimmed.len() <= max_len {
        trimmed.to_string()
    } else {
        format!("{}…", &trimmed[..max_len])
    }
}

fn classify_agent_build_error(err: anyhow::Error, provider: &str, model: &str) -> ValidationError {
    let msg = err.to_string();
    let lower = msg.to_lowercase();

    let (kind, hint) = if lower.contains("url") || lower.contains("parse") {
        (
            ValidationErrorKind::NetworkFailure,
            "Check that api_base in config.json is a valid URL.".to_string(),
        )
    } else {
        (
            ValidationErrorKind::Unknown,
            "Check config.json and provider documentation.".to_string(),
        )
    };

    ValidationError {
        kind,
        message: msg,
        hint,
        provider: Some(provider.to_string()),
        model: Some(model.to_string()),
    }
}

/// Classify a PromptError into a user-friendly ValidationError.
///
/// Uses substring matching on the error's Display output, since rig wraps
/// provider-specific messages into opaque error strings.
pub(crate) fn classify_prompt_error(
    err: rig::completion::PromptError,
    provider: &str,
    model: &str,
) -> ValidationError {
    let msg = err.to_string();
    let lower = msg.to_lowercase();

    let (kind, hint) = classify_error_text(&lower);

    ValidationError {
        kind,
        message: msg,
        hint,
        provider: Some(provider.to_string()),
        model: Some(model.to_string()),
    }
}

/// Core classification logic on lowercased error text. Separated for testability.
fn classify_error_text(lower: &str) -> (ValidationErrorKind, String) {
    if lower.contains("401")
        || lower.contains("unauthorized")
        || lower.contains("authentication")
        || lower.contains("invalid api key")
        || lower.contains("invalid x-api-key")
        || lower.contains("incorrect api key")
        || (lower.contains("permission") && lower.contains("denied"))
        || lower.contains("forbidden")
        || lower.contains("403")
    {
        return (
            ValidationErrorKind::AuthFailure,
            "Check that api_key in config.json is valid and not expired.".to_string(),
        );
    }

    if lower.contains("model")
        && (lower.contains("not found")
            || lower.contains("not_found")
            || lower.contains("does not exist")
            || lower.contains("not available"))
        || lower.contains("404") && lower.contains("model")
        || lower.contains("no such model")
        || lower.contains("invalid model")
    {
        return (
            ValidationErrorKind::ModelNotAvailable,
            "Verify the model name is correct and available for your provider.".to_string(),
        );
    }

    if lower.contains("connect")
        || lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("dns")
        || lower.contains("network")
        || lower.contains("unreachable")
        || lower.contains("connection refused")
        || lower.contains("connection reset")
        || lower.contains("broken pipe")
        || lower.contains("tls")
        || lower.contains("ssl")
        || lower.contains("certificate")
        || lower.contains("resolve")
    {
        return (
            ValidationErrorKind::NetworkFailure,
            "Check network connectivity and that api_base URL is reachable.".to_string(),
        );
    }

    (
        ValidationErrorKind::Unknown,
        "Check config.json and provider documentation.".to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_auth_401() {
        let (kind, _) = classify_error_text("providerError: 401 unauthorized");
        assert_eq!(kind, ValidationErrorKind::AuthFailure);
    }

    #[test]
    fn test_classify_auth_invalid_key() {
        let (kind, _) = classify_error_text("providerError: invalid api key provided");
        assert_eq!(kind, ValidationErrorKind::AuthFailure);
    }

    #[test]
    fn test_classify_auth_incorrect_key() {
        let (kind, _) = classify_error_text("providerError: incorrect api key");
        assert_eq!(kind, ValidationErrorKind::AuthFailure);
    }

    #[test]
    fn test_classify_auth_forbidden() {
        let (kind, _) = classify_error_text("providerError: 403 forbidden");
        assert_eq!(kind, ValidationErrorKind::AuthFailure);
    }

    #[test]
    fn test_classify_auth_permission_denied() {
        let (kind, _) = classify_error_text("providerError: permission denied for this resource");
        assert_eq!(kind, ValidationErrorKind::AuthFailure);
    }

    #[test]
    fn test_classify_model_not_found() {
        let (kind, _) = classify_error_text("providerError: model 'gpt-999' not found");
        assert_eq!(kind, ValidationErrorKind::ModelNotAvailable);
    }

    #[test]
    fn test_classify_model_does_not_exist() {
        let (kind, _) = classify_error_text("providerError: the model `gpt-5` does not exist");
        assert_eq!(kind, ValidationErrorKind::ModelNotAvailable);
    }

    #[test]
    fn test_classify_no_such_model() {
        let (kind, _) = classify_error_text("error: no such model: llama-99b");
        assert_eq!(kind, ValidationErrorKind::ModelNotAvailable);
    }

    #[test]
    fn test_classify_invalid_model() {
        let (kind, _) = classify_error_text("error: invalid model identifier");
        assert_eq!(kind, ValidationErrorKind::ModelNotAvailable);
    }

    #[test]
    fn test_classify_network_connection_refused() {
        let (kind, _) = classify_error_text("httperror: connection refused");
        assert_eq!(kind, ValidationErrorKind::NetworkFailure);
    }

    #[test]
    fn test_classify_network_timeout() {
        let (kind, _) = classify_error_text("httperror: request timed out");
        assert_eq!(kind, ValidationErrorKind::NetworkFailure);
    }

    #[test]
    fn test_classify_network_dns() {
        let (kind, _) = classify_error_text("httperror: dns resolution failed");
        assert_eq!(kind, ValidationErrorKind::NetworkFailure);
    }

    #[test]
    fn test_classify_network_tls() {
        let (kind, _) = classify_error_text("httperror: tls handshake failure");
        assert_eq!(kind, ValidationErrorKind::NetworkFailure);
    }

    #[test]
    fn test_classify_network_unreachable() {
        let (kind, _) = classify_error_text("httperror: host unreachable");
        assert_eq!(kind, ValidationErrorKind::NetworkFailure);
    }

    #[test]
    fn test_classify_unknown_error() {
        let (kind, _) = classify_error_text("some completely unexpected error text");
        assert_eq!(kind, ValidationErrorKind::Unknown);
    }

    #[test]
    fn test_classify_unknown_has_hint() {
        let (_, hint) = classify_error_text("some random error");
        assert!(hint.contains("config.json"));
    }

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate_response("Hello!", 80), "Hello!");
    }

    #[test]
    fn test_truncate_long() {
        let long = "a".repeat(100);
        let result = truncate_response(&long, 80);
        assert!(result.ends_with('…'));
        assert!(result.len() <= 84);
    }

    #[test]
    fn test_truncate_trims_whitespace() {
        assert_eq!(truncate_response("  hello  ", 80), "hello");
    }

    #[test]
    fn test_validation_error_kind_display() {
        assert_eq!(
            ValidationErrorKind::ConfigLoad.to_string(),
            "configuration error"
        );
        assert_eq!(
            ValidationErrorKind::AuthFailure.to_string(),
            "authentication error"
        );
        assert_eq!(
            ValidationErrorKind::NetworkFailure.to_string(),
            "network/endpoint error"
        );
        assert_eq!(
            ValidationErrorKind::ModelNotAvailable.to_string(),
            "model not available"
        );
        assert_eq!(ValidationErrorKind::Unknown.to_string(), "unknown error");
    }

    #[test]
    fn test_classify_config_error_missing_file() {
        let lower = "config file not found at: /home/user/.config/hi/config.json";
        let (kind, _) = classify_error_text(lower);
        assert_eq!(kind, ValidationErrorKind::Unknown);
    }

    #[test]
    fn test_classify_config_error_parse_failure() {
        let lower = "failed to parse config.json: expected `,` or `}`";
        let (kind, _) = classify_error_text(lower);
        assert_eq!(kind, ValidationErrorKind::Unknown);
    }
}
