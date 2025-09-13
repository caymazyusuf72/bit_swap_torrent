# BitSwapTorrent 🔄

Modern, güvenli ve ölçeklenebilir P2P dosya paylaşım protokolü - Rust ile yazılmış BitTorrent alternatifi.

![BitSwapTorrent](https://img.shields.io/badge/Language-Rust-orange)
![Version](https://img.shields.io/badge/Version-0.1.0-blue)
![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-green)

## ✨ Özellikler (MVP v0.1)

- 🌐 **Trackerless DHT**: Kademlia DHT ile merkezi olmayan peer discovery
- 🔒 **SHA-256 Doğrulama**: Her parça için kriptografik doğrulama
- 📦 **Modern Metadata**: JSON tabanlı .bwt dosya formatı
- 🚀 **Async/Tokio**: Yüksek performanslı asenkron I/O
- 🎨 **Renkli CLI**: Kullanıcı dostu komut satırı arayüzü
- 💾 **Resume Desteği**: Kesintisiz indirme/upload
- 🔄 **P2P Mesajlaşma**: BitTorrent benzeri wire protocol

## 🚀 Kurulum

### Gereksinimler

- Rust 1.70+ 
- Cargo

### Build

```bash
git clone https://github.com/caymazeryusuf72/bitswaptorrent
cd bit_swap_torrent
cargo build --release
```

Binary dosya: `target/release/bwt`

## 📖 Kullanım

### Torrent Oluştur

```bash
# Tek dosya için
bwt add dosya.iso

# Klasör için 
bwt add ./proje-klasoru

# Custom piece size ve tracker ile
bwt add dosya.bin --piece-len 2048 --announce http://tracker.example.com:8080/announce

# Oluşturduktan sonra seed'e başla
bwt add dosya.iso --seed
```

### Seed (Paylaş)

```bash
bwt seed dosya.bwt

# Custom port ile
bwt seed dosya.bwt --port 6882
```

### Download (İndir)

```bash
# .bwt dosyası ile
bwt download dosya.bwt

# Magnet link ile (TODO)
bwt download "magnet:?xt=urn:btih:..."

# Custom çıkış dizini ile
bwt download dosya.bwt --output ./indirilecek-yer
```

### Durum ve Yönetim

```bash
# Aktif torrent'ları listele
bwt status

# Bağlı peer'ları göster
bwt peers

# Magnet link oluştur
bwt share dosya.bwt --magnet

# Konfigürasyon
bwt config show
```

## 🏗️ Mimari

### Core Modüller

```
bit-swap-core/
├── metadata.rs      # .bwt dosya formatı
├── protocol.rs      # Wire protocol mesajlaşma
├── storage.rs       # Piece-based dosya yönetimi  
├── dht.rs          # Kademlia DHT implementation
├── transport.rs     # TCP/WebSocket transport
├── peer.rs         # Peer management
├── scheduler.rs     # Piece selection algoritmaları
├── crypto.rs       # SHA-256 ve crypto utils
└── error.rs        # Hata tipleri
```

### .bwt Dosya Formatı

```json
{
  "name": "dosya.iso",
  "created_by": "BitSwapTorrent/0.1.0",
  "created_at": "2025-09-13T09:00:00Z",
  "piece_length": 1048576,
  "pieces": ["sha256-hash-1", "sha256-hash-2", "..."],
  "files": [
    {
      "path": ["dosya.iso"],
      "length": 4294967296
    }
  ],
  "info_hash": "torrent-info-hash-sha256",
  "trackers": [],
  "web_seed": []
}
```

## 🔧 Geliştirici Rehberi

### Test

```bash
# Unit testler
cargo test

# Integration testler
cargo test --test '*'

# Benchmark'lar
cargo bench
```

### Linting

```bash
cargo clippy
cargo fmt
```

### Feature'lar

Şu anda MVP (v0.1) aşamasındayız. Gelecek sürümler için:

**V1 (v0.5):**
- [ ] NAT traversal (STUN/UPnP)
- [ ] UTP/QUIC transport
- [ ] Gelişmiş scheduler (rarest-first)
- [ ] Web seeder desteği
- [ ] GUI (Electron/egui)

**V2 (v1.0+):**
- [ ] Encrypted transports (Noise)
- [ ] Incentive mekanizmaları
- [ ] Content addressing (IPFS benzeri)
- [ ] Lightning Network entegrasyonu

## 🤝 Katkıda Bulunma

1. Fork edin
2. Feature branch oluşturun (`git checkout -b feature/amazing-feature`)
3. Commit edin (`git commit -m 'Add amazing feature'`)
4. Push edin (`git push origin feature/amazing-feature`)
5. Pull Request açın

## 📄 Lisans

Bu proje MIT veya Apache-2.0 lisansları altında dağıtılmaktadır.

## ⚠️ Yasal Uyarı

Bu yazılım yalnızca yasal içerikler için kullanılmalıdır. Telifli içerik paylaşımı yasa dışıdır ve kullanıcının sorumluluğundadır.

## 🔗 Bağlantılar


- [Crates.io](https://crates.io/crates/bit-swap-torrent)
- [Docs.rs](https://docs.rs/bit-swap-core)

---

BitSwapTorrent ile güvenli ve modern P2P dosya paylaşımı! 🚀
