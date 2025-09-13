//! BitSwapTorrent CLI
//!
//! Komut satırı arayüzü

use anyhow::Result;
use bit_swap_core::{
    metadata::BitSwapMetadata, BitSwapClient, ClientConfig, NAME, VERSION,
};
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;
use tracing::{info, warn};

/// BitSwapTorrent CLI - Modern P2P file sharing
#[derive(Parser)]
#[command(name = "bwt")]
#[command(version = VERSION)]
#[command(about = "BitSwapTorrent - Güvenli ve modern P2P dosya paylaşımı", long_about = None)]
struct Cli {
    /// Verbose logging
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    
    /// Config file path
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Dosya veya klasör için .bwt metadata oluştur
    Add {
        /// Dosya veya klasör yolu
        path: PathBuf,
        
        /// Parça boyutu (KB) [varsayılan: 1024]
        #[arg(long, default_value = "1024")]
        piece_len: u32,
        
        /// Tracker URL'leri
        #[arg(long, action = clap::ArgAction::Append)]
        announce: Vec<String>,
        
        /// Oluşturulduktan sonra seed'e başla
        #[arg(long)]
        seed: bool,
        
        /// Çıktı .bwt dosya adı
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Torrent seed et
    Seed {
        /// .bwt metadata dosyası
        metadata: PathBuf,
        
        /// Listen portu [varsayılan: 6881]
        #[arg(short, long, default_value = "6881")]
        port: u16,
    },
    
    /// Torrent indir
    Download {
        /// .bwt dosyası veya magnet link
        source: String,
        
        /// İndirme dizini [varsayılan: ./downloads]
        #[arg(short, long, default_value = "./downloads")]
        output: PathBuf,
        
        /// Listen portu [varsayılan: 6881]
        #[arg(short, long, default_value = "6881")]
        port: u16,
    },
    
    /// Aktif torrent durumlarını göster
    Status,
    
    /// Bağlı peer'ları listele
    Peers,
    
    /// Torrent'i durdur
    Stop {
        /// Torrent ID veya info hash
        id: String,
    },
    
    /// Magnet link oluştur veya göster
    Share {
        /// .bwt metadata dosyası
        metadata: PathBuf,
        
        /// Magnet linki yazdır
        #[arg(long)]
        magnet: bool,
    },
    
    /// Web UI başlat
    Webui {
        #[command(subcommand)]
        action: WebuiCommands,
    },
    
    /// Konfigürasyon yönetimi
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum WebuiCommands {
    /// Web UI'yi başlat
    Start {
        /// Web UI portu [varsayılan: 8080]
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Konfigürasyonu göster
    Show,
    /// Konfigürasyonu düzenle
    Edit,
    /// Konfigürasyonu sıfırla
    Reset,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Logging seviyesini ayarla
    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(format!("bit_swap_torrent={},bit_swap_core={}", log_level, log_level))
        .with_target(false)
        .init();

    // Welcome message
    println!("{} {}", NAME.bright_cyan().bold(), VERSION.bright_green());
    println!();

    match &cli.command {
        Commands::Add {
            path,
            piece_len,
            announce,
            seed,
            output,
        } => {
            handle_add(path, *piece_len * 1024, announce, *seed, output.as_ref()).await?;
        }
        
        Commands::Seed { metadata, port } => {
            handle_seed(metadata, *port).await?;
        }
        
        Commands::Download { source, output, port } => {
            handle_download(source, output, *port).await?;
        }
        
        Commands::Status => {
            handle_status().await?;
        }
        
        Commands::Peers => {
            handle_peers().await?;
        }
        
        Commands::Stop { id } => {
            handle_stop(id).await?;
        }
        
        Commands::Share { metadata, magnet } => {
            handle_share(metadata, *magnet).await?;
        }
        
        Commands::Webui { action } => {
            match action {
                WebuiCommands::Start { port } => {
                    handle_webui_start(*port).await?;
                }
            }
        }
        
        Commands::Config { action } => {
            match action {
                ConfigCommands::Show => handle_config_show().await?,
                ConfigCommands::Edit => handle_config_edit().await?,
                ConfigCommands::Reset => handle_config_reset().await?,
            }
        }
    }

    Ok(())
}

async fn handle_add(
    path: &PathBuf,
    piece_length: u32,
    trackers: &[String],
    start_seed: bool,
    output: Option<&PathBuf>,
) -> Result<()> {
    println!("📦 {} için metadata oluşturuluyor...", path.display().to_string().bright_yellow());
    
    let metadata = if path.is_file() {
        BitSwapMetadata::from_file(path, piece_length).await?
    } else if path.is_dir() {
        BitSwapMetadata::from_directory(path, piece_length).await?
    } else {
        anyhow::bail!("Belirtilen yol dosya veya klasör değil: {}", path.display());
    };

    // Tracker'ları ekle
    let mut final_metadata = metadata;
    final_metadata.trackers.extend_from_slice(trackers);

    // Dosya adını belirle
    let output_path = match output {
        Some(path) => path.clone(),
        None => {
            let mut name = final_metadata.name.clone();
            name.push_str(".bwt");
            PathBuf::from(name)
        }
    };

    // Metadata dosyasını kaydet
    final_metadata.save_to_file(&output_path).await?;

    println!("✅ Metadata oluşturuldu: {}", output_path.display().to_string().bright_green());
    println!("   📊 {} parça, {} toplam boyut", 
        final_metadata.piece_count().to_string().bright_blue(),
        format_bytes(final_metadata.total_size()).bright_blue()
    );
    println!("   🔗 Info hash: {}", final_metadata.info_hash.bright_magenta());

    if start_seed {
        println!("🌱 Seed'e başlanıyor...");
        handle_seed(&output_path, 6881).await?;
    }

    Ok(())
}

async fn handle_seed(metadata_path: &PathBuf, port: u16) -> Result<()> {
    let metadata = BitSwapMetadata::load_from_file(metadata_path).await?;
    
    println!("🌱 {} seed'e başlanıyor...", metadata.name.bright_yellow());
    println!("   🔗 Info hash: {}", metadata.info_hash.bright_magenta());
    println!("   🌐 Port: {}", port.to_string().bright_blue());

    let config = ClientConfig {
        ..Default::default()
    };

    let mut client = BitSwapClient::new(config)?;
    client.start(port).await?;

    println!("✅ Seed başlatıldı. Ctrl+C ile durdurun.");
    
    // Seed loop - gerçek implementasyonda burada peer connections olacak
    tokio::signal::ctrl_c().await?;
    println!("\n🛑 Seed durduruluyor...");

    Ok(())
}

async fn handle_download(source: &str, output_dir: &PathBuf, port: u16) -> Result<()> {
    let metadata = if source.starts_with("magnet:") {
        println!("🧲 Magnet linkinden metadata alınıyor...");
        // TODO: Magnet link parsing ve metadata fetch
        anyhow::bail!("Magnet link desteği henüz implement edilmedi");
    } else {
        BitSwapMetadata::load_from_file(source).await?
    };

    println!("⬇️ {} indiriliyor...", metadata.name.bright_yellow());
    println!("   📁 Hedef dizin: {}", output_dir.display().to_string().bright_cyan());
    println!("   📊 {} parça, {} toplam boyut", 
        metadata.piece_count().to_string().bright_blue(),
        format_bytes(metadata.total_size()).bright_blue()
    );

    // Output dizinini oluştur
    tokio::fs::create_dir_all(output_dir).await?;

    let config = ClientConfig {
        ..Default::default()
    };

    let mut client = BitSwapClient::new(config)?;
    client.start(port).await?;

    println!("✅ İndirme başlatıldı. Ctrl+C ile durdurun.");
    
    // Download loop - gerçek implementasyonda burada download logic olacak
    tokio::signal::ctrl_c().await?;
    println!("\n🛑 İndirme durduruluyor...");

    Ok(())
}

async fn handle_status() -> Result<()> {
    println!("📊 {}", "Aktif Torrent Durumları".bright_cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    
    // TODO: Gerçek torrent durumlarını göster
    println!("Henüz aktif torrent yok.");
    
    Ok(())
}

async fn handle_peers() -> Result<()> {
    println!("👥 {}", "Bağlı Peer'lar".bright_cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    
    // TODO: Gerçek peer listesini göster
    println!("Henüz bağlı peer yok.");
    
    Ok(())
}

async fn handle_stop(id: &str) -> Result<()> {
    println!("🛑 Torrent durduruluyor: {}", id.bright_yellow());
    
    // TODO: Torrent'i durdur
    println!("⚠️ Bu özellik henüz implement edilmedi.");
    
    Ok(())
}

async fn handle_share(metadata_path: &PathBuf, show_magnet: bool) -> Result<()> {
    let metadata = BitSwapMetadata::load_from_file(metadata_path).await?;
    
    println!("🔗 {}", "Paylaşım Bilgileri".bright_cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!("📝 İsim: {}", metadata.name.bright_yellow());
    println!("🔗 Info hash: {}", metadata.info_hash.bright_magenta());
    
    if show_magnet {
        let magnet = metadata.to_magnet_url();
        println!("🧲 Magnet link:");
        println!("   {}", magnet.bright_blue());
    }
    
    Ok(())
}

async fn handle_webui_start(port: u16) -> Result<()> {
    println!("🌐 Web UI başlatılıyor: http://localhost:{}", port.to_string().bright_blue());
    
    // TODO: Web UI server başlat
    println!("⚠️ Web UI henüz implement edilmedi.");
    
    Ok(())
}

async fn handle_config_show() -> Result<()> {
    println!("⚙️ {}", "Konfigürasyon".bright_cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    
    let config = ClientConfig::default();
    println!("Max peers: {}", config.max_peers.to_string().bright_blue());
    println!("Piece size: {}", format_bytes(config.piece_size as u64).bright_blue());
    println!("Data directory: {}", config.data_dir.display().to_string().bright_cyan());
    
    Ok(())
}

async fn handle_config_edit() -> Result<()> {
    println!("⚙️ Konfigürasyon düzenleme henüz implement edilmedi.");
    Ok(())
}

async fn handle_config_reset() -> Result<()> {
    println!("⚙️ Konfigürasyon sıfırlama henüz implement edilmedi.");
    Ok(())
}

/// Byte'ları okunabilir formatta göster
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }
}
