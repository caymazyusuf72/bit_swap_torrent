//! BitSwapTorrent hata tipleri

use thiserror::Error;

/// BitSwapTorrent ana hata tipi
#[derive(Error, Debug)]
pub enum BitSwapError {
    /// IO hatası
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Serialization hatası
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// Transport hatası
    #[error("Transport error: {0}")]
    Transport(String),
    
    /// DHT hatası
    #[error("DHT error: {0}")]
    Dht(String),
    
    /// Peer hatası
    #[error("Peer error: {0}")]
    Peer(String),
    
    /// Protocol hatası
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    /// Metadata hatası
    #[error("Metadata error: {0}")]
    Metadata(String),
    
    /// Storage hatası
    #[error("Storage error: {0}")]
    Storage(String),
    
    /// Crypto/hash hatası
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    /// Konfigürasyon hatası
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Piece verification hatası
    #[error("Piece verification failed: piece {piece_index}, expected hash {expected}, got {actual}")]
    PieceVerificationFailed {
        piece_index: u32,
        expected: String,
        actual: String,
    },
    
    /// Invalid torrent file
    #[error("Invalid torrent file: {0}")]
    InvalidTorrent(String),
    
    /// Peer disconnected
    #[error("Peer disconnected: {peer_id}")]
    PeerDisconnected { peer_id: String },
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded for peer {peer_id}")]
    RateLimitExceeded { peer_id: String },
    
    /// Invalid piece index
    #[error("Invalid piece index: {index}, total pieces: {total}")]
    InvalidPieceIndex { index: u32, total: u32 },
    
    /// Generic error
    #[error("Error: {0}")]
    Other(String),
}

/// Result type for BitSwapTorrent operations
pub type BitSwapResult<T> = Result<T, BitSwapError>;

impl From<String> for BitSwapError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}

impl From<&str> for BitSwapError {
    fn from(s: &str) -> Self {
        Self::Other(s.to_string())
    }
}
