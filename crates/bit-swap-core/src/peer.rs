//! Peer management
//!
//! Peer connection ve state management

use crate::error::{BitSwapError, BitSwapResult};
use std::net::SocketAddr;
use uuid::Uuid;

/// Peer bilgileri
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer ID
    pub peer_id: Uuid,
    /// Peer address
    pub addr: SocketAddr,
    /// Connection durumu
    pub connected: bool,
    /// Upload/download istatistikleri
    pub stats: PeerStats,
}

/// Peer istatistikleri
#[derive(Debug, Clone)]
pub struct PeerStats {
    /// Uploaded bytes
    pub uploaded: u64,
    /// Downloaded bytes  
    pub downloaded: u64,
    /// Connection timestamp
    pub connected_at: Option<u64>,
}

impl Default for PeerStats {
    fn default() -> Self {
        Self {
            uploaded: 0,
            downloaded: 0,
            connected_at: None,
        }
    }
}

/// Peer manager
#[derive(Debug)]
pub struct PeerManager {
    peers: std::collections::HashMap<Uuid, PeerInfo>,
}

impl PeerManager {
    /// Yeni peer manager oluştur
    pub fn new() -> Self {
        Self {
            peers: std::collections::HashMap::new(),
        }
    }

    /// Peer ekle
    pub fn add_peer(&mut self, peer_id: Uuid, addr: SocketAddr) {
        let peer_info = PeerInfo {
            peer_id,
            addr,
            connected: false,
            stats: PeerStats::default(),
        };
        self.peers.insert(peer_id, peer_info);
    }

    /// Peer'ı kaldır
    pub fn remove_peer(&mut self, peer_id: &Uuid) -> Option<PeerInfo> {
        self.peers.remove(peer_id)
    }

    /// Tüm peer'ları al
    pub fn get_peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().collect()
    }
}

impl Default for PeerManager {
    fn default() -> Self {
        Self::new()
    }
}
