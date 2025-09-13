//! Piece scheduler
//!
//! Piece selection ve download scheduling algoritmaları

use crate::error::{BitSwapError, BitSwapResult};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Piece selection stratejisi
#[derive(Debug, Clone, Copy)]
pub enum SelectionStrategy {
    /// Sequential (sıralı indirme)
    Sequential,
    /// Rarest first (en nadir piece'leri önce)
    RarestFirst,
    /// Random
    Random,
}

/// Download scheduler
#[derive(Debug)]
pub struct PieceScheduler {
    /// Selection stratejisi
    strategy: SelectionStrategy,
    /// Requesting pieces (peer_id -> piece_index set)
    requesting: HashMap<Uuid, HashSet<u32>>,
    /// Piece rarity map (piece_index -> peer count)
    piece_availability: HashMap<u32, u32>,
}

impl PieceScheduler {
    /// Yeni scheduler oluştur
    pub fn new(strategy: SelectionStrategy) -> Self {
        Self {
            strategy,
            requesting: HashMap::new(),
            piece_availability: HashMap::new(),
        }
    }

    /// Piece availability güncelle
    pub fn update_peer_pieces(&mut self, peer_id: Uuid, pieces: &[u32]) {
        for &piece_index in pieces {
            *self.piece_availability.entry(piece_index).or_insert(0) += 1;
        }
    }

    /// İndirilecek piece seç
    pub fn select_piece(
        &self,
        peer_pieces: &[u32],
        local_pieces: &[u32],
    ) -> Option<u32> {
        let local_set: HashSet<u32> = local_pieces.iter().copied().collect();
        let available_pieces: Vec<u32> = peer_pieces
            .iter()
            .copied()
            .filter(|piece| !local_set.contains(piece))
            .collect();

        if available_pieces.is_empty() {
            return None;
        }

        match self.strategy {
            SelectionStrategy::Sequential => {
                available_pieces.into_iter().min()
            }
            SelectionStrategy::RarestFirst => {
                available_pieces
                    .into_iter()
                    .min_by_key(|&piece| self.piece_availability.get(&piece).copied().unwrap_or(0))
            }
            SelectionStrategy::Random => {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                available_pieces.choose(&mut rng).copied()
            }
        }
    }

    /// Request piece
    pub fn request_piece(&mut self, peer_id: Uuid, piece_index: u32) {
        self.requesting.entry(peer_id).or_default().insert(piece_index);
    }

    /// Complete piece
    pub fn complete_piece(&mut self, peer_id: Uuid, piece_index: u32) {
        if let Some(pieces) = self.requesting.get_mut(&peer_id) {
            pieces.remove(&piece_index);
        }
    }
}

impl Default for PieceScheduler {
    fn default() -> Self {
        Self::new(SelectionStrategy::RarestFirst)
    }
}
