use thiserror::Error;

#[derive(Error, Debug)]
pub enum PeerTlsError {
    #[error("TLS error: {0}")]
    Tls(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Certificate error: {0}")]
    Certificate(String),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, PeerTlsError>;

impl From<String> for PeerTlsError {
    fn from(msg: String) -> Self {
        PeerTlsError::Other(msg)
    }
}

impl From<&str> for PeerTlsError {
    fn from(msg: &str) -> Self {
        PeerTlsError::Other(msg.to_string())
    }
} 