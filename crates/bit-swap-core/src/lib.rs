//! BitSwapTorrent Core Library
//!
//! Bu kütüphane BitSwapTorrent P2P dosya paylaşım protokolünün
//! temel implementasyonunu içerir.

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]

pub mod metadata;
pub mod dht;
pub mod transport;
pub mod protocol;
pub mod storage;
pub mod peer;
pub mod scheduler;
pub mod error;
pub mod crypto;

use anyhow::Result;
use std::net::SocketAddr;
use uuid::Uuid;

/// BitSwapTorrent client'ının ana yapısı
#[derive(Debug)]
pub struct BitSwapClient {
    /// Unique client identifier
    pub peer_id: Uuid,
    /// Local listening address
    pub local_addr: Option<SocketAddr>,
    /// Client configuration
    pub config: ClientConfig,
}

/// Client konfigürasyon parametreleri
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Maximum number of peer connections
    pub max_peers: usize,
    /// Maximum upload rate (bytes/second, 0 = unlimited)
    pub max_upload_rate: u64,
    /// Maximum download rate (bytes/second, 0 = unlimited)
    pub max_download_rate: u64,
    /// Piece size in bytes (must be power of 2)
    pub piece_size: u32,
    /// DHT bootstrap nodes
    pub bootstrap_nodes: Vec<SocketAddr>,
    /// Enable DHT
    pub enable_dht: bool,
    /// Data directory for storage
    pub data_dir: std::path::PathBuf,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            max_peers: 50,
            max_upload_rate: 0, // unlimited
            max_download_rate: 0, // unlimited
            piece_size: 1024 * 1024, // 1MB
            bootstrap_nodes: vec![
                "67.205.187.89:6881".parse().unwrap(),
                "82.221.103.244:6881".parse().unwrap(),
            ],
            enable_dht: true,
            data_dir: dirs::data_local_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("bitswaptorrent"),
        }
    }
}

impl BitSwapClient {
    /// Yeni bir BitSwapTorrent client oluştur
    pub fn new(config: ClientConfig) -> Result<Self> {
        let peer_id = Uuid::new_v4();
        
        // Data directory oluştur
        std::fs::create_dir_all(&config.data_dir)?;
        
        Ok(Self {
            peer_id,
            local_addr: None,
            config,
        })
    }

    /// Client'ı başlat ve belirli bir portta dinlemeye başla
    pub async fn start(&mut self, port: u16) -> Result<()> {
        tracing::info!("BitSwapTorrent client starting on port {}", port);
        
        // Transport layer'ı başlat
        let addr = format!("0.0.0.0:{}", port).parse()?;
        self.local_addr = Some(addr);
        
        tracing::info!("Client started with peer_id: {}", self.peer_id);
        Ok(())
    }
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "BitSwapTorrent";
pub const USER_AGENT: &str = concat!("BitSwapTorrent/", env!("CARGO_PKG_VERSION"));
