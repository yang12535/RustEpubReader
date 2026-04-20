//! Keystore management for the secure P2P sharing feature.
const SERVICE: &str = "com.epub.reader.sharing";

/// Store the private key PEM in the OS keychain.
pub fn store_private_key(device_id: &str, pem: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, device_id).map_err(|e| e.to_string())?;
    entry.set_password(pem).map_err(|e| e.to_string())
}

/// Load the private key PEM from the OS keychain.
pub fn load_private_key(device_id: &str) -> Option<String> {
    let entry = keyring::Entry::new(SERVICE, device_id).ok()?;
    entry.get_password().ok()
}

/// Delete the private key from the OS keychain.
pub fn delete_private_key(device_id: &str) {
    if let Ok(entry) = keyring::Entry::new(SERVICE, device_id) {
        let _ = entry.delete_credential();
    }
}

// ── Generic credential helpers ──

const GITHUB_SERVICE: &str = "com.epub.reader.github";

pub fn store_github_token(token: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(GITHUB_SERVICE, "oauth_token").map_err(|e| e.to_string())?;
    entry.set_password(token).map_err(|e| e.to_string())
}

pub fn load_github_token() -> Option<String> {
    let entry = keyring::Entry::new(GITHUB_SERVICE, "oauth_token").ok()?;
    entry.get_password().ok()
}

pub fn delete_github_token() {
    if let Ok(entry) = keyring::Entry::new(GITHUB_SERVICE, "oauth_token") {
        let _ = entry.delete_credential();
    }
}
