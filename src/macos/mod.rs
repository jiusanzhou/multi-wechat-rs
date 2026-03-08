pub mod config;
pub mod macho;
pub mod patcher;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("app not found: {0}")]
    AppNotFound(String),

    #[error("unsupported version: {0}")]
    UnsupportedVersion(String),

    #[error("config load error: {0}")]
    ConfigLoad(String),

    #[error("patch error: {0}")]
    Patch(String),

    #[error("codesign error: {0}")]
    Codesign(String),

    #[error("io error: {0}")]
    Io(String),
}
