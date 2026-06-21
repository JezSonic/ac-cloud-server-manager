use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerManagerError {
    #[error("SSH Error: {0}")]
    SshError(#[from] ssh2::Error),

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON Serialization Error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("INI Parsing Error: {0}")]
    IniError(#[from] ini::ParseError),

    #[error("Server Setup Error: {0}")]
    SetupError(String),

    #[error("Validation Error: {0}")]
    ValidationError(String),

    #[error("SteamCMD Error: {0}")]
    SteamCmdError(String),

    #[error("Crypto Error: {0}")]
    CryptoError(String),
}
