//! BitSwapTorrent metadata (.bwt) formatı ve işlemleri

use crate::error::{BitSwapError, BitSwapResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// .bwt dosya içeriği
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitSwapMetadata {
    /// Torrent adı
    pub name: String,
    /// Oluşturan client bilgisi
    pub created_by: String,
    /// Oluşturulma tarihi (RFC3339 format)
    pub created_at: String,
    /// Parça boyutu (bytes)
    pub piece_length: u32,
    /// Her parça için SHA-256 hash (hex format)
    pub pieces: Vec<String>,
    /// Dosya listesi
    pub files: Vec<FileEntry>,
    /// Info hash (metadata'nın hash'i)
    pub info_hash: String,
    /// Tracker listesi (opsiyonel)
    #[serde(default)]
    pub trackers: Vec<String>,
    /// Web seed URL'leri (opsiyonel)
    #[serde(default)]
    pub web_seed: Vec<String>,
    /// Ek metadata (opsiyonel)
    #[serde(default)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Dosya girdisi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Dosya yolu (klasör/dosya hierarchy)
    pub path: Vec<String>,
    /// Dosya boyutu (bytes)
    pub length: u64,
    /// Dosya hash'i (opsiyonel, bütün dosya için)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_hash: Option<String>,
}

impl BitSwapMetadata {
    /// Yeni bir metadata oluştur
    pub fn new(name: String, piece_length: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let created_at = chrono::DateTime::from_timestamp(now as i64, 0)
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();

        Self {
            name,
            created_by: format!("BitSwapTorrent/{}", crate::VERSION),
            created_at,
            piece_length,
            pieces: Vec::new(),
            files: Vec::new(),
            info_hash: String::new(),
            trackers: Vec::new(),
            web_seed: Vec::new(),
            extra: std::collections::HashMap::new(),
        }
    }

    /// Tek dosya için metadata oluştur
    pub async fn from_file<P: AsRef<Path>>(
        file_path: P,
        piece_length: u32,
    ) -> BitSwapResult<Self> {
        let path = file_path.as_ref();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| BitSwapError::Metadata("Invalid file name".to_string()))?;

        let metadata = tokio::fs::metadata(path).await?;
        let file_size = metadata.len();

        let mut meta = Self::new(file_name.to_string(), piece_length);
        
        // Dosya girdisi ekle
        meta.files.push(FileEntry {
            path: vec![file_name.to_string()],
            length: file_size,
            file_hash: None,
        });

        // Parçalara böl ve hash hesapla
        meta.calculate_pieces(path).await?;
        
        // Info hash hesapla
        meta.calculate_info_hash()?;

        Ok(meta)
    }

    /// Klasör için metadata oluştur (recursive)
    pub async fn from_directory<P: AsRef<Path>>(
        dir_path: P,
        piece_length: u32,
    ) -> BitSwapResult<Self> {
        let path = dir_path.as_ref();
        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| BitSwapError::Metadata("Invalid directory name".to_string()))?;

        let mut meta = Self::new(dir_name.to_string(), piece_length);
        
        // Recursive olarak dosyaları topla
        collect_files_recursive(path, path, &mut meta.files).await?;
        
        if meta.files.is_empty() {
            return Err(BitSwapError::Metadata("No files found in directory".to_string()));
        }

        // Pieces hesapla
        meta.calculate_pieces_from_files(path).await?;
        
        // Info hash hesapla
        meta.calculate_info_hash()?;

        Ok(meta)
    }

    /// Parça hash'lerini hesapla (tek dosya için)
    async fn calculate_pieces<P: AsRef<Path>>(&mut self, file_path: P) -> BitSwapResult<()> {
        use tokio::io::AsyncReadExt;
        
        let mut file = tokio::fs::File::open(file_path).await?;
        let mut buffer = vec![0u8; self.piece_length as usize];
        let mut piece_index = 0u32;

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            let piece_data = &buffer[..bytes_read];
            let hash = Sha256::digest(piece_data);
            self.pieces.push(hex::encode(hash));
            
            piece_index += 1;
        }

        tracing::info!("Calculated {} pieces for file", piece_index);
        Ok(())
    }

    /// Multiple dosyalardan pieces hesapla
    async fn calculate_pieces_from_files<P: AsRef<Path>>(&mut self, base_path: P) -> BitSwapResult<()> {
        use tokio::io::AsyncReadExt;
        
        let mut hasher = Sha256::new();
        let mut current_piece_size = 0u32;
        
        for file_entry in &self.files {
            let file_path = base_path.as_ref().join(file_entry.path.join("/"));
            let mut file = tokio::fs::File::open(file_path).await?;
            let mut buffer = vec![0u8; 8192]; // 8KB buffer

            loop {
                let bytes_read = file.read(&mut buffer).await?;
                if bytes_read == 0 {
                    break;
                }

                let mut remaining = bytes_read;
                let mut offset = 0;

                while remaining > 0 {
                    let space_in_piece = self.piece_length - current_piece_size;
                    let to_take = remaining.min(space_in_piece as usize);
                    
                    hasher.update(&buffer[offset..offset + to_take]);
                    current_piece_size += to_take as u32;
                    
                    if current_piece_size == self.piece_length {
                        // Piece tamamlandı
                        let hash = hasher.finalize_reset();
                        self.pieces.push(hex::encode(hash));
                        current_piece_size = 0;
                    }
                    
                    remaining -= to_take;
                    offset += to_take;
                }
            }
        }

        // Son kalan parça varsa
        if current_piece_size > 0 {
            let hash = hasher.finalize();
            self.pieces.push(hex::encode(hash));
        }

        tracing::info!("Calculated {} pieces from {} files", self.pieces.len(), self.files.len());
        Ok(())
    }

    /// Info hash hesapla
    fn calculate_info_hash(&mut self) -> BitSwapResult<()> {
        // Info struct oluştur (canonical format için)
        let info = InfoDict {
            name: &self.name,
            piece_length: self.piece_length,
            pieces: &self.pieces,
            files: &self.files,
        };

        let info_json = serde_json::to_string(&info)?;
        let hash = Sha256::digest(info_json.as_bytes());
        self.info_hash = hex::encode(hash);
        
        Ok(())
    }

    /// JSON'dan metadata yükle
    pub fn from_json(json_str: &str) -> BitSwapResult<Self> {
        let metadata: BitSwapMetadata = serde_json::from_str(json_str)?;
        
        // Temel validasyonlar
        if metadata.name.is_empty() {
            return Err(BitSwapError::Metadata("Empty name".to_string()));
        }
        
        if metadata.pieces.is_empty() {
            return Err(BitSwapError::Metadata("No pieces".to_string()));
        }
        
        if metadata.files.is_empty() {
            return Err(BitSwapError::Metadata("No files".to_string()));
        }

        Ok(metadata)
    }

    /// JSON'a serialize et
    pub fn to_json(&self) -> BitSwapResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Dosyaya kaydet
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> BitSwapResult<()> {
        let json = self.to_json()?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    /// Dosyadan yükle
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> BitSwapResult<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_json(&content)
    }

    /// Magnet URL oluştur
    pub fn to_magnet_url(&self) -> String {
        let mut magnet = format!("magnet:?xt=urn:btih:{}", self.info_hash);
        magnet.push_str(&format!("&dn={}", urlencoding::encode(&self.name)));
        
        for tracker in &self.trackers {
            magnet.push_str(&format!("&tr={}", urlencoding::encode(tracker)));
        }
        
        for ws in &self.web_seed {
            magnet.push_str(&format!("&ws={}", urlencoding::encode(ws)));
        }
        
        magnet
    }

    /// Toplam boyut hesapla
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.length).sum()
    }

    /// Piece sayısı
    pub fn piece_count(&self) -> u32 {
        self.pieces.len() as u32
    }
}

/// Info dictionary (info hash hesabı için canonical format)
#[derive(Serialize)]
struct InfoDict<'a> {
    name: &'a str,
    piece_length: u32,
    pieces: &'a Vec<String>,
    files: &'a Vec<FileEntry>,
}

/// Recursive olarak dosyaları topla
fn collect_files_recursive<'a>(
    current_path: &'a Path,
    base_path: &'a Path,
    files: &'a mut Vec<FileEntry>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = BitSwapResult<()>> + 'a>> {
    Box::pin(async move {
        let mut entries = tokio::fs::read_dir(current_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;

            if metadata.is_file() {
                // Relative path hesapla
                let rel_path = path.strip_prefix(base_path)
                    .map_err(|_| BitSwapError::Metadata("Invalid path".to_string()))?;
                
                let path_components: Vec<String> = rel_path
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect();

                files.push(FileEntry {
                    path: path_components,
                    length: metadata.len(),
                    file_hash: None,
                });
            } else if metadata.is_dir() {
                // Recursive dive
                collect_files_recursive(&path, base_path, files).await?;
            }
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_metadata_creation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, b"Hello, BitSwapTorrent!").await.unwrap();

        let metadata = BitSwapMetadata::from_file(&file_path, 1024).await.unwrap();
        
        assert_eq!(metadata.name, "test.txt");
        assert!(!metadata.pieces.is_empty());
        assert_eq!(metadata.files.len(), 1);
        assert_eq!(metadata.files[0].length, 21);
    }

    #[test]
    fn test_magnet_url() {
        let mut metadata = BitSwapMetadata::new("test".to_string(), 1024);
        metadata.info_hash = "abcd1234".to_string();
        metadata.trackers.push("http://tracker.example.com:8080/announce".to_string());
        
        let magnet = metadata.to_magnet_url();
        assert!(magnet.contains("magnet:?xt=urn:btih:abcd1234"));
        assert!(magnet.contains("dn=test"));
        assert!(magnet.contains("tr=http%3A//tracker.example.com%3A8080/announce"));
    }
}
