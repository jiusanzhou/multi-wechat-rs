#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
fn main() {
    windows::run();
}

#[cfg(target_os = "macos")]
fn main() {
    macos::cli::run();
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn main() {
    eprintln!("This tool only supports Windows and macOS.");
    std::process::exit(1);
}
