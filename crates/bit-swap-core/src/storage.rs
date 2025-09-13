//! BitSwapTorrent storage system
//!
//! Parça tabanlı dosya depolama ve yönetim sistemi

use crate::error::{BitSwapError, BitSwapResult};
use crate::metadata::{BitSwapMetadata, FileEntry};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use uuid::Uuid;

/// Piece durumu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PieceState {
    /// Parça mevcut değil
    Missing,
    /// İndiriliyor
    Downloading,
    /// İndirildi ama doğrulanmadı
    Downloaded,
    /// Doğrulandı ve kullanılabilir
    Verified,
    /// Bozuk (hash eşleşmiyor)
    Corrupted,
}

/// Storage manager - dosya parçalarını yönetir
#[derive(Debug)]
pub struct StorageManager {
    /// Torrent metadata
    metadata: BitSwapMetadata,
    /// Storage dizini
    storage_dir: PathBuf,
    /// Piece durumları
    piece_states: HashMap<u32, PieceState>,
    /// Açık dosya handles (cache)
    file_handles: HashMap<PathBuf, File>,
    /// Resume veritabanı
    db: sled::Db,
}

/// Piece bilgileri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieceInfo {
    /// Piece index
    pub index: u32,
    /// Piece hash
    pub hash: String,
    /// Piece boyutu (son piece daha küçük olabilir)
    pub size: u32,
    /// Bu piece'in hangi dosya(lar)da olduğu
    pub file_ranges: Vec<FileRange>,
}

/// Dosya içinde piece'in konumu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRange {
    /// Dosya path
    pub file_path: PathBuf,
    /// Dosya içinde başlangıç offset
    pub file_offset: u64,
    /// Bu piece'den kaç byte bu dosyada
    pub length: u32,
}

/// Resume data yapısı
#[derive(Debug, Serialize, Deserialize)]
pub struct ResumeData {
    /// Torrent info hash
    pub info_hash: String,
    /// Piece durumları
    pub piece_states: HashMap<u32, PieceState>,
    /// Son güncelleme zamanı
    pub last_updated: u64,
    /// Toplam indirilen bytes
    pub downloaded_bytes: u64,
    /// Toplam upload bytes
    pub uploaded_bytes: u64,
}

impl StorageManager {
    /// Yeni storage manager oluştur
    pub async fn new(
        metadata: BitSwapMetadata,
        storage_dir: PathBuf,
    ) -> BitSwapResult<Self> {
        // Storage dizini oluştur
        tokio::fs::create_dir_all(&storage_dir).await?;
        
        // Resume veritabanı aç
        let db_path = storage_dir.join("resume.db");
        let db = sled::open(db_path)
            .map_err(|e| BitSwapError::Storage(format!("Failed to open database: {}", e)))?;

        let mut storage = Self {
            metadata,
            storage_dir,
            piece_states: HashMap::new(),
            file_handles: HashMap::new(),
            db,
        };

        // Piece durumlarını yükle veya başlat
        storage.load_or_initialize_pieces().await?;
        
        Ok(storage)
    }

    /// Piece durumlarını yükle veya başlat
    async fn load_or_initialize_pieces(&mut self) -> BitSwapResult<()> {
        let info_hash = &self.metadata.info_hash;
        
        // Resume data varsa yükle
        if let Ok(Some(resume_bytes)) = self.db.get(info_hash.as_bytes()) {
            match bincode::deserialize::<ResumeData>(&resume_bytes) {
                Ok(resume_data) => {
                    tracing::info!("Resume data loaded for torrent {}", info_hash);
                    self.piece_states = resume_data.piece_states;
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("Failed to deserialize resume data: {}", e);
                }
            }
        }

        // Resume data yok, tüm piece'leri missing olarak başlat
        for i in 0..self.metadata.piece_count() {
            self.piece_states.insert(i, PieceState::Missing);
        }

        // Mevcut dosyaları kontrol et ve piece durumlarını güncelle
        self.verify_existing_pieces().await?;

        tracing::info!("Initialized {} pieces", self.metadata.piece_count());
        Ok(())
    }

    /// Mevcut dosyaları kontrol et ve piece durumlarını güncelle
    async fn verify_existing_pieces(&mut self) -> BitSwapResult<()> {
        let piece_infos = self.calculate_piece_info();
        
        for (piece_index, piece_info) in piece_infos {
            if self.verify_piece_on_disk(piece_index, &piece_info).await? {
                self.piece_states.insert(piece_index, PieceState::Verified);
            }
        }

        let verified_count = self.piece_states.values()
            .filter(|&&state| state == PieceState::Verified)
            .count();
        
        tracing::info!("Found {} verified pieces on disk", verified_count);
        Ok(())
    }

    /// Piece bilgilerini hesapla (hangi dosyalarda hangi offset'lerde)
    fn calculate_piece_info(&self) -> HashMap<u32, PieceInfo> {
        let mut piece_infos = HashMap::new();
        let mut current_offset = 0u64;
        let piece_size = self.metadata.piece_length;

        for (piece_index, piece_hash) in self.metadata.pieces.iter().enumerate() {
            let piece_index = piece_index as u32;
            let mut file_ranges = Vec::new();
            let mut remaining_piece_size = piece_size;
            let mut piece_offset = current_offset;

            // Bu piece hangi dosya(lar)da
            for file_entry in &self.metadata.files {
                if piece_offset >= current_offset + file_entry.length {
                    continue;
                }

                let file_path = self.storage_dir.join(file_entry.path.join(std::path::MAIN_SEPARATOR_STR));
                let file_start = current_offset;
                let file_end = current_offset + file_entry.length;
                
                let range_start = piece_offset.max(file_start);
                let range_end = (piece_offset + remaining_piece_size as u64).min(file_end);
                
                if range_start < range_end {
                    let file_offset = range_start - file_start;
                    let length = (range_end - range_start) as u32;
                    
                    file_ranges.push(FileRange {
                        file_path,
                        file_offset,
                        length,
                    });

                    remaining_piece_size = remaining_piece_size.saturating_sub(length);
                    piece_offset = range_end;

                    if remaining_piece_size == 0 {
                        break;
                    }
                }

                current_offset += file_entry.length;
            }

            // Son piece küçük olabilir
            let actual_size = piece_size - remaining_piece_size;
            
            piece_infos.insert(piece_index, PieceInfo {
                index: piece_index,
                hash: piece_hash.clone(),
                size: actual_size,
                file_ranges,
            });

            current_offset = piece_offset;
        }

        piece_infos
    }

    /// Disk'te piece'i doğrula
    async fn verify_piece_on_disk(&mut self, piece_index: u32, piece_info: &PieceInfo) -> BitSwapResult<bool> {
        let mut piece_data = Vec::with_capacity(piece_info.size as usize);
        
        // Tüm file range'leri oku
        for file_range in &piece_info.file_ranges {
            // Dosya yoksa piece mevcut değil
            if !file_range.file_path.exists() {
                return Ok(false);
            }

            let file_handle = self.get_file_handle(&file_range.file_path, false).await?;
            let mut buffer = vec![0u8; file_range.length as usize];
            
            let mut file = file_handle;
            file.seek(SeekFrom::Start(file_range.file_offset)).await?;
            file.read_exact(&mut buffer).await?;
            
            piece_data.extend_from_slice(&buffer);
        }

        // Hash'i doğrula
        let calculated_hash = hex::encode(Sha256::digest(&piece_data));
        Ok(calculated_hash == piece_info.hash)
    }

    /// Dosya handle al (cache'den veya aç)
    async fn get_file_handle(&mut self, path: &Path, create: bool) -> BitSwapResult<&mut File> {
        if !self.file_handles.contains_key(path) {
            // Dizini oluştur
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            let file = if create {
                OpenOptions::new()
                    .create(true)
                    .read(true)
                    .write(true)
                    .open(path)
                    .await?
            } else {
                File::open(path).await?
            };

            self.file_handles.insert(path.to_owned(), file);
        }

        Ok(self.file_handles.get_mut(path).unwrap())
    }

    /// Piece yaz
    pub async fn write_piece(&mut self, piece_index: u32, data: &[u8]) -> BitSwapResult<()> {
        let piece_infos = self.calculate_piece_info();
        let piece_info = piece_infos.get(&piece_index)
            .ok_or_else(|| BitSwapError::InvalidPieceIndex {
                index: piece_index,
                total: self.metadata.piece_count(),
            })?;

        // Data boyutunu kontrol et
        if data.len() != piece_info.size as usize {
            return Err(BitSwapError::Storage(format!(
                "Piece size mismatch: expected {}, got {}",
                piece_info.size,
                data.len()
            )));
        }

        // Hash'i doğrula
        let calculated_hash = hex::encode(Sha256::digest(data));
        if calculated_hash != piece_info.hash {
            self.piece_states.insert(piece_index, PieceState::Corrupted);
            return Err(BitSwapError::PieceVerificationFailed {
                piece_index,
                expected: piece_info.hash.clone(),
                actual: calculated_hash,
            });
        }

        // Dosyalara yaz
        let mut data_offset = 0usize;
        
        for file_range in &piece_info.file_ranges {
            let file_handle = self.get_file_handle(&file_range.file_path, true).await?;
            let chunk = &data[data_offset..data_offset + file_range.length as usize];
            
            file_handle.seek(SeekFrom::Start(file_range.file_offset)).await?;
            file_handle.write_all(chunk).await?;
            file_handle.flush().await?;
            
            data_offset += file_range.length as usize;
        }

        // Durumu güncelle
        self.piece_states.insert(piece_index, PieceState::Verified);
        
        // Resume data kaydet
        self.save_resume_data().await?;
        
        tracing::debug!("Piece {} written and verified", piece_index);
        Ok(())
    }

    /// Piece oku
    pub async fn read_piece(&mut self, piece_index: u32) -> BitSwapResult<Vec<u8>> {
        let state = self.piece_states.get(&piece_index)
            .copied()
            .unwrap_or(PieceState::Missing);

        if state != PieceState::Verified {
            return Err(BitSwapError::Storage(format!(
                "Piece {} not available (state: {:?})",
                piece_index, state
            )));
        }

        let piece_infos = self.calculate_piece_info();
        let piece_info = piece_infos.get(&piece_index)
            .ok_or_else(|| BitSwapError::InvalidPieceIndex {
                index: piece_index,
                total: self.metadata.piece_count(),
            })?;

        let mut piece_data = Vec::with_capacity(piece_info.size as usize);

        // File range'leri oku
        for file_range in &piece_info.file_ranges {
            let file_handle = self.get_file_handle(&file_range.file_path, false).await?;
            let mut buffer = vec![0u8; file_range.length as usize];
            
            file_handle.seek(SeekFrom::Start(file_range.file_offset)).await?;
            file_handle.read_exact(&mut buffer).await?;
            
            piece_data.extend_from_slice(&buffer);
        }

        Ok(piece_data)
    }

    /// Piece durumunu al
    pub fn get_piece_state(&self, piece_index: u32) -> PieceState {
        self.piece_states.get(&piece_index)
            .copied()
            .unwrap_or(PieceState::Missing)
    }

    /// Piece durumunu değiştir
    pub fn set_piece_state(&mut self, piece_index: u32, state: PieceState) {
        self.piece_states.insert(piece_index, state);
    }

    /// Tamamlanma yüzdesi
    pub fn completion_percentage(&self) -> f32 {
        let verified_count = self.piece_states.values()
            .filter(|&&state| state == PieceState::Verified)
            .count();
        
        (verified_count as f32 / self.metadata.piece_count() as f32) * 100.0
    }

    /// Eksik piece'leri al
    pub fn get_missing_pieces(&self) -> Vec<u32> {
        (0..self.metadata.piece_count())
            .filter(|&index| {
                matches!(
                    self.get_piece_state(index),
                    PieceState::Missing | PieceState::Corrupted
                )
            })
            .collect()
    }

    /// Mevcut piece'leri al
    pub fn get_available_pieces(&self) -> Vec<u32> {
        (0..self.metadata.piece_count())
            .filter(|&index| self.get_piece_state(index) == PieceState::Verified)
            .collect()
    }

    /// Resume data kaydet
    pub async fn save_resume_data(&self) -> BitSwapResult<()> {
        let resume_data = ResumeData {
            info_hash: self.metadata.info_hash.clone(),
            piece_states: self.piece_states.clone(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            downloaded_bytes: self.calculate_downloaded_bytes(),
            uploaded_bytes: 0, // TODO: Track upload stats
        };

        let serialized = bincode::serialize(&resume_data)
            .map_err(|e| BitSwapError::Storage(format!("Failed to serialize resume data: {}", e)))?;

        self.db.insert(self.metadata.info_hash.as_bytes(), serialized)
            .map_err(|e| BitSwapError::Storage(format!("Failed to save resume data: {}", e)))?;

        Ok(())
    }

    /// İndirilmiş byte'ları hesapla
    fn calculate_downloaded_bytes(&self) -> u64 {
        let mut bytes = 0u64;
        let piece_infos = self.calculate_piece_info();
        
        for (&piece_index, &state) in &self.piece_states {
            if state == PieceState::Verified {
                if let Some(piece_info) = piece_infos.get(&piece_index) {
                    bytes += piece_info.size as u64;
                }
            }
        }
        
        bytes
    }

    /// Storage'ı kapat (dosyaları flush et)
    pub async fn close(&mut self) -> BitSwapResult<()> {
        // Tüm dosyaları flush et
        for (_, file) in &mut self.file_handles {
            file.flush().await?;
        }
        
        // Resume data kaydet
        self.save_resume_data().await?;
        
        // DB'yi flush et
        self.db.flush()
            .map_err(|e| BitSwapError::Storage(format!("Failed to flush database: {}", e)))?;

        tracing::info!("Storage manager closed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::FileEntry;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_storage_manager() {
        let temp_dir = TempDir::new().unwrap();
        let storage_dir = temp_dir.path().to_owned();

        // Test metadata oluştur
        let mut metadata = BitSwapMetadata::new("test".to_string(), 32);
        metadata.files.push(FileEntry {
            path: vec!["test.txt".to_string()],
            length: 64,
            file_hash: None,
        });
        metadata.pieces = vec![
            "a".repeat(64), // İlk 32 byte
            "b".repeat(64), // Son 32 byte
        ];

        let mut storage = StorageManager::new(metadata, storage_dir).await.unwrap();
        
        // Başlangıçta tüm piece'ler missing olmalı
        assert_eq!(storage.get_piece_state(0), PieceState::Missing);
        assert_eq!(storage.get_piece_state(1), PieceState::Missing);
        
        // Completion 0% olmalı
        assert_eq!(storage.completion_percentage(), 0.0);
        
        // Missing pieces listesi doğru olmalı
        assert_eq!(storage.get_missing_pieces(), vec![0, 1]);
        
        storage.close().await.unwrap();
    }
}
