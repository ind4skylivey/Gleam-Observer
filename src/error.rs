use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("GPU error: {0}")]
    Gpu(String),

    #[error("Metrics error: {0}")]
    Metrics(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("TUI error: {0}")]
    Tui(String),

    #[error("System error: {0}")]
    System(String),

    #[error("Daemon error: {0}")]
    Daemon(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, Error>;
