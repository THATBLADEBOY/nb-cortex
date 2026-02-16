//! API key management commands using the OS keychain.
//!
//! These commands let the frontend manage API keys for LLM services
//! without ever exposing the actual key values to the webview.

use serde::{Deserialize, Serialize};
use specta::Type;

const KEYCHAIN_SERVICE_PREFIX: &str = "com.THATBLADEBOY.cortex";

/// Known API key services and their display metadata.
const KNOWN_SERVICES: &[(&str, &str)] = &[
    ("openai", "OpenAI"),
    ("anthropic", "Anthropic"),
    ("google", "Google AI"),
];

/// Entry describing an API key service and whether a key is stored.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ApiKeyEntry {
    /// Machine-readable service identifier (e.g., "openai").
    pub service: String,
    /// Human-readable display name (e.g., "OpenAI").
    pub display_name: String,
    /// Whether a key is currently stored in the keychain.
    pub has_key: bool,
}

/// Builds the full keychain service string for a given service ID.
fn keychain_service(service: &str) -> String {
    format!("{KEYCHAIN_SERVICE_PREFIX}.api-key.{service}")
}

/// Store an API key in the OS keychain.
#[tauri::command]
#[specta::specta]
pub fn set_api_key(service: String, key: String) -> Result<(), String> {
    log::info!("Storing API key for service: {service}");

    let entry = keyring::Entry::new(&keychain_service(&service), "api-key")
        .map_err(|e| format!("Failed to create keychain entry: {e}"))?;

    entry
        .set_password(&key)
        .map_err(|e| format!("Failed to store API key: {e}"))?;

    log::info!("API key stored successfully for service: {service}");
    Ok(())
}

/// Remove an API key from the OS keychain.
#[tauri::command]
#[specta::specta]
pub fn delete_api_key(service: String) -> Result<(), String> {
    log::info!("Deleting API key for service: {service}");

    let entry = keyring::Entry::new(&keychain_service(&service), "api-key")
        .map_err(|e| format!("Failed to create keychain entry: {e}"))?;

    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete API key: {e}"))?;

    log::info!("API key deleted for service: {service}");
    Ok(())
}

/// Check if an API key exists in the OS keychain (never exposes the actual key).
#[tauri::command]
#[specta::specta]
pub fn has_api_key(service: String) -> Result<bool, String> {
    let entry = keyring::Entry::new(&keychain_service(&service), "api-key")
        .map_err(|e| format!("Failed to create keychain entry: {e}"))?;

    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(format!("Failed to check API key: {e}")),
    }
}

/// List all known API key services with their storage status.
#[tauri::command]
#[specta::specta]
pub fn list_api_key_services() -> Result<Vec<ApiKeyEntry>, String> {
    let mut entries = Vec::new();

    for (service, display_name) in KNOWN_SERVICES {
        let has_key = has_api_key(service.to_string()).unwrap_or(false);
        entries.push(ApiKeyEntry {
            service: service.to_string(),
            display_name: display_name.to_string(),
            has_key,
        });
    }

    Ok(entries)
}
