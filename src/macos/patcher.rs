//! High-level patch orchestration:
//! read WeChat version → match config → patch binary → codesign.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::macos::config;
use crate::macos::macho;
use crate::macos::Error;

/// Resolved paths for a WeChat.app bundle.
pub struct WeChatApp {
    pub app_path: PathBuf,
    pub binary_path: PathBuf,
    pub plist_path: PathBuf,
}

impl WeChatApp {
    pub fn new(app_path: &Path) -> Self {
        Self {
            app_path: app_path.to_path_buf(),
            binary_path: app_path.join("Contents/MacOS/WeChat"),
            plist_path: app_path.join("Contents/Info.plist"),
        }
    }

    /// Verify that the app bundle exists with expected files.
    pub fn validate(&self) -> Result<(), Error> {
        if !self.app_path.exists() {
            return Err(Error::AppNotFound(self.app_path.display().to_string()));
        }
        if !self.binary_path.exists() {
            return Err(Error::AppNotFound(format!(
                "binary not found: {}",
                self.binary_path.display()
            )));
        }
        if !self.plist_path.exists() {
            return Err(Error::AppNotFound(format!(
                "Info.plist not found: {}",
                self.plist_path.display()
            )));
        }
        Ok(())
    }
}

/// Read WeChat version from Info.plist (CFBundleVersion).
pub fn read_version(app: &WeChatApp) -> Result<String, Error> {
    let output = Command::new("defaults")
        .args(["read", &app.plist_path.to_string_lossy(), "CFBundleVersion"])
        .output()
        .map_err(|e| Error::Io(format!("run defaults: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Io(format!("defaults read failed: {}", stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Execute the full patch flow.
pub fn patch(app: &WeChatApp, config_path: Option<&Path>) -> Result<(), Error> {
    app.validate()?;

    // 1. Read version
    let version = read_version(app)?;
    println!("WeChat version: {}", version);

    // 2. Load config and match version
    let configs = match config_path {
        Some(path) => config::load_from_file(path)?,
        None => config::load_embedded()?,
    };

    let matched = config::find_for_version(&configs, &version).ok_or_else(|| {
        Error::UnsupportedVersion(format!(
            "{} (supported: {})",
            version,
            configs
                .iter()
                .map(|c| c.version.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    })?;

    println!("Matched config for version {}", matched.version);

    // 3. Collect all entries from all targets
    let entries: Vec<_> = matched
        .targets
        .iter()
        .flat_map(|t| t.entries.clone())
        .collect();
    println!("Applying {} patch entries...", entries.len());

    // 4. Patch binary
    let results = macho::patch_binary(&app.binary_path, &entries)?;
    for r in &results {
        println!(
            "  [{}] VA=0x{:x} patched at file offset 0x{:x}",
            r.arch, r.va, r.file_offset
        );
    }

    // 5. Re-sign
    println!("Re-signing...");
    codesign(app)?;
    println!("Done!");

    Ok(())
}

/// List supported versions from config.
pub fn list_versions(
    app: &WeChatApp,
    config_path: Option<&Path>,
) -> Result<(Option<String>, Vec<String>), Error> {
    let current = if app.app_path.exists() {
        read_version(app).ok()
    } else {
        None
    };

    let configs = match config_path {
        Some(path) => config::load_from_file(path)?,
        None => config::load_embedded()?,
    };

    let versions: Vec<String> = configs.iter().map(|c| c.version.clone()).collect();
    Ok((current, versions))
}

/// Remove signature, re-sign ad-hoc, and clear extended attributes.
fn codesign(app: &WeChatApp) -> Result<(), Error> {
    let app_path = app.app_path.to_string_lossy();

    run_cmd("codesign", &["--remove-sign", &app_path])?;
    run_cmd("codesign", &["--force", "--deep", "--sign", "-", &app_path])?;
    run_cmd("xattr", &["-cr", &app_path])?;

    Ok(())
}

fn run_cmd(program: &str, args: &[&str]) -> Result<(), Error> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| Error::Io(format!("run {}: {}", program, e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Codesign(format!("{} failed: {}", program, stderr)));
    }

    Ok(())
}
