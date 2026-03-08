use serde::Deserialize;
use std::path::Path;

/// Embedded default config from WeChatTweak
const DEFAULT_CONFIG_JSON: &str = include_str!("config.json");

/// A version-specific patch configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct PatchConfig {
    pub version: String,
    pub targets: Vec<Target>,
}

/// A patch target (e.g. "revoke", "multiInstance").
#[derive(Debug, Clone, Deserialize)]
pub struct Target {
    #[allow(dead_code)]
    pub identifier: String,
    pub entries: Vec<Entry>,
}

/// A single patch entry for a specific architecture.
#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    pub arch: String,
    /// Virtual address as hex string (e.g. "103dba3d0")
    pub addr: String,
    /// Machine code as hex string (e.g. "00008052C0035FD6")
    pub asm: String,
}

impl Entry {
    /// Parse the hex addr string to u64.
    pub fn addr_u64(&self) -> Result<u64, std::num::ParseIntError> {
        u64::from_str_radix(&self.addr, 16)
    }

    /// Parse the hex asm string to bytes.
    pub fn asm_bytes(&self) -> Option<Vec<u8>> {
        hex_to_bytes(&self.asm)
    }
}

/// Load configs from an external JSON file.
pub fn load_from_file(path: &Path) -> Result<Vec<PatchConfig>, crate::macos::Error> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| crate::macos::Error::ConfigLoad(format!("{}: {}", path.display(), e)))?;
    serde_json::from_str(&data).map_err(|e| crate::macos::Error::ConfigLoad(e.to_string()))
}

/// Load the embedded default config.
pub fn load_embedded() -> Result<Vec<PatchConfig>, crate::macos::Error> {
    serde_json::from_str(DEFAULT_CONFIG_JSON)
        .map_err(|e| crate::macos::Error::ConfigLoad(e.to_string()))
}

/// Find config matching a specific WeChat version.
pub fn find_for_version(configs: &[PatchConfig], version: &str) -> Option<PatchConfig> {
    configs.iter().find(|c| c.version == version).cloned()
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_embedded() {
        let configs = load_embedded().unwrap();
        assert!(!configs.is_empty());
        // Should have the known version 31927
        assert!(configs.iter().any(|c| c.version == "31927"));
    }

    #[test]
    fn test_hex_to_bytes() {
        assert_eq!(
            hex_to_bytes("00008052C0035FD6"),
            Some(vec![0x00, 0x00, 0x80, 0x52, 0xC0, 0x03, 0x5F, 0xD6])
        );
        assert_eq!(hex_to_bytes(""), Some(vec![]));
        assert_eq!(hex_to_bytes("0"), None); // odd length
    }

    #[test]
    fn test_entry_parsing() {
        let configs = load_embedded().unwrap();
        let config = configs.iter().find(|c| c.version == "31927").unwrap();
        let revoke = config
            .targets
            .iter()
            .find(|t| t.identifier == "revoke")
            .unwrap();
        let entry = &revoke.entries[0];
        assert_eq!(entry.arch, "arm64");
        assert_eq!(entry.addr_u64().unwrap(), 0x103dba3d0);
        assert_eq!(
            entry.asm_bytes().unwrap(),
            vec![0x00, 0x00, 0x80, 0x52, 0xC0, 0x03, 0x5F, 0xD6]
        );
    }
}
