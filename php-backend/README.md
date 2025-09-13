# 🚀 BitSwapTorrent - Gerçek P2P Dosya Paylaşım Sistemi

## 🎯 Ne Bu?
Bu **GERÇEK** bir dosya paylaşım sistemi! Hiçbir mockup değil, tamamen çalışan PHP backend'i var:

- ✅ **Her dosya türünü** yükleyebilirsiniz (video, müzik, oyun, yazılım...)
- ✅ **SHA-256** hash ile dosya doğrulaması
- ✅ **Magnet link** oluşturma ve paylaşma
- ✅ **Global indirme** linkler
- ✅ **Gerçek zamanlı istatistikler**
- ✅ **SQLite database** ile dosya takibi
- ✅ **Responsive web arayüzü**

## 📋 Kurulum

### Gereksinimler:
- PHP 7.4+ (SQLite desteği ile)
- Web server (Apache/Nginx) VEYA `php -S`

### Hızlı Kurulum:
```bash
# 1. Dosyaları web server klasörüne kopyala
cp -r php-backend/* /var/www/html/bitswap/

# VEYA sadece test için:
cd php-backend
php -S localhost:8080
```

### Tarayıcıda açın:
```
http://localhost:8080
VEYA
http://yourdomain.com/bitswap/
```

## 🔥 Nasıl Kullanılır?

### 1️⃣ Dosya Yükle:
- Web sitesini aç
- "Dosya Yükle" butonuna tıkla
- İstediğin dosyayı seç
- Yükleme tamamlanınca **magnet link** panoya kopyalanır!

### 2️⃣ Dosya Paylaş:
- Magnet linki arkadaşına gönder
- VEYA direkt indirme linkini paylaş
- Dosya global olarak erişilebilir!

### 3️⃣ Dosya İndir:
- "Dosyalar" sekmesinden mevcut dosyaları gör
- "İndir" butonuna tıkla
- Dosya hemen indirilmeye başlar!

## 🌐 API Endpoints

```php
POST /api/upload.php       // Dosya yükle
GET  /api/download.php?hash=XXX  // Dosya indir
GET  /api/files.php?action=list  // Dosya listesi
GET  /api/files.php?action=stats // İstatistikler
```

## 💾 Database Schema

SQLite database otomatik oluşturulur:

```sql
CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    file_hash TEXT UNIQUE,      -- SHA-256 hash
    original_name TEXT,         -- Orijinal dosya adı
    file_path TEXT,            -- Server'daki konum
    file_size INTEGER,         -- Byte cinsinden
    mime_type TEXT,            -- Dosya türü
    upload_time DATETIME,      -- Yüklenme zamanı
    download_count INTEGER,    -- İndirme sayısı
    uploader_ip TEXT          -- Yükleyen IP
);

CREATE TABLE peers (
    id INTEGER PRIMARY KEY,
    file_hash TEXT,           -- Hangi dosya
    peer_ip TEXT,            -- Peer IP adresi
    last_seen DATETIME       -- Son görülme
);
```

## 🔒 Güvenlik Özellikleri

- **SHA-256** ile dosya bütünlüğü kontrolü
- **Unique hash** ile duplicate engellemesi  
- **CORS** headers ile güvenli API
- **SQL injection** koruması
- **File type** doğrulaması

## 🌍 Production Deploy

### Apache ile:
```apache
<VirtualHost *:80>
    ServerName bitswap.yourdomain.com
    DocumentRoot /var/www/html/bitswap
    
    # Upload limiti
    php_value upload_max_filesize 1G
    php_value post_max_size 1G
    php_value max_execution_time 300
</VirtualHost>
```

### Nginx ile:
```nginx
server {
    listen 80;
    server_name bitswap.yourdomain.com;
    root /var/www/html/bitswap;
    index index.php;

    # Upload limiti
    client_max_body_size 1G;
    
    location ~ \.php$ {
        fastcgi_pass unix:/run/php/php8.0-fpm.sock;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }
}
```

## 📈 Özellikler

### ✅ Tamamlanan:
- [x] Dosya yükleme/indirme
- [x] SHA-256 hash doğrulama
- [x] Magnet link oluşturma
- [x] Web UI ile dosya yönetimi
- [x] Gerçek zamanlı istatistikler
- [x] Responsive tasarım
- [x] SQLite database
- [x] API endpoints

### 🔄 Geliştirme Aşamasında:
- [ ] Chunk'lı indirme (büyük dosyalar için)
- [ ] BitTorrent protokolü uyumluluğu
- [ ] DHT peer discovery
- [ ] Encrypted transfers
- [ ] User authentication
- [ ] File search/filtering
- [ ] Rate limiting

## 🆘 Sorun Giderme

### Dosya yüklenmiyor?
- `uploads/` klasörü yazılabilir mi kontrol et
- PHP upload limitlerini kontrol et (`php.ini`)
- Server disk alanını kontrol et

### Database hatası?
- SQLite extension yüklü mü? (`php -m | grep sqlite`)
- Database dosyası yazılabilir mi?

### API çalışmıyor?
- CORS headers aktif mi?
- URL rewriting doğru mu?
- PHP error log'larını kontrol et

## 🤝 Katkı

Bu proje açık kaynak! Katkıda bulunmak için:
1. Fork et
2. Feature branch oluştur
3. Commit et
4. Pull request aç

## 📄 Lisans

MIT License - İstediğiniz gibi kullanın!

---

**⚡ Bu sistem GERÇEK dosya paylaşımı yapar! Test edin ve görün! ⚡**
