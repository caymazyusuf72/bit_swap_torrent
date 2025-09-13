#!/usr/bin/env python3
"""
BitSwapTorrent - Basit ve Çalışan Dosya Paylaşım Sistemi
Flask tabanlı - Kolay kurulum
"""

try:
    from flask import Flask, render_template_string, request, jsonify, send_file, redirect
    import os
    import hashlib
    import json
    import sqlite3
    from datetime import datetime
    import mimetypes
    
    print("✅ Flask yüklü! Server başlatılıyor...")
except ImportError:
    print("❌ Flask yüklü değil!")
    print("🔧 Kurulum için: pip install flask")
    exit(1)

app = Flask(__name__)
app.config['MAX_CONTENT_LENGTH'] = 1024 * 1024 * 1024  # 1GB limit

# Upload directory
UPLOAD_DIR = 'uploads'
DB_FILE = 'bitswap.db'

os.makedirs(UPLOAD_DIR, exist_ok=True)

def init_db():
    """Initialize database"""
    conn = sqlite3.connect(DB_FILE)
    conn.execute('''
        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_hash TEXT UNIQUE NOT NULL,
            original_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            upload_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            download_count INTEGER DEFAULT 0,
            uploader_ip TEXT
        )
    ''')
    conn.commit()
    conn.close()

def get_db():
    """Get database connection"""
    conn = sqlite3.connect(DB_FILE)
    conn.row_factory = sqlite3.Row
    return conn

@app.route('/')
def index():
    """Ana sayfa"""
    return render_template_string(HTML_TEMPLATE)

@app.route('/api/upload', methods=['POST'])
def upload_file():
    """Dosya yükleme"""
    try:
        if 'file' not in request.files:
            return jsonify({'success': False, 'message': 'Dosya seçilmedi'})
        
        file = request.files['file']
        if file.filename == '':
            return jsonify({'success': False, 'message': 'Dosya seçilmedi'})
        
        # Dosya içeriğini oku
        file_content = file.read()
        original_name = file.filename
        file_size = len(file_content)
        
        # Hash hesapla
        file_hash = hashlib.sha256(file_content).hexdigest()
        
        # Database'de var mı kontrol et
        conn = get_db()
        existing = conn.execute('SELECT * FROM files WHERE file_hash = ?', (file_hash,)).fetchone()
        
        if existing:
            conn.close()
            return jsonify({
                'success': True,
                'message': 'Dosya zaten mevcut',
                'hash': file_hash,
                'share_url': f"{request.host_url}download/{file_hash}"
            })
        
        # Dosyayı kaydet
        filename = f"{file_hash}_{original_name}"
        file_path = os.path.join(UPLOAD_DIR, filename)
        
        with open(file_path, 'wb') as f:
            f.write(file_content)
        
        # Database'e ekle
        uploader_ip = request.remote_addr
        conn.execute('''
            INSERT INTO files (file_hash, original_name, file_path, file_size, uploader_ip)
            VALUES (?, ?, ?, ?, ?)
        ''', (file_hash, original_name, file_path, file_size, uploader_ip))
        conn.commit()
        conn.close()
        
        return jsonify({
            'success': True,
            'message': 'Dosya başarıyla yüklendi',
            'hash': file_hash,
            'original_name': original_name,
            'size': file_size,
            'share_url': f"{request.host_url}download/{file_hash}"
        })
        
    except Exception as e:
        return jsonify({'success': False, 'message': str(e)})

@app.route('/api/files')
def get_files():
    """Dosya listesi ve istatistikler"""
    action = request.args.get('action', 'list')
    
    conn = get_db()
    
    if action == 'stats':
        row = conn.execute('SELECT COUNT(*) as total_files, SUM(file_size) as total_size, SUM(download_count) as total_downloads FROM files').fetchone()
        stats = {
            'total_files': row['total_files'] or 0,
            'total_size': row['total_size'] or 0,
            'total_downloads': row['total_downloads'] or 0
        }
        conn.close()
        return jsonify({'success': True, 'stats': stats})
    
    else:
        files = []
        rows = conn.execute('SELECT * FROM files ORDER BY upload_time DESC LIMIT 50').fetchall()
        for row in rows:
            files.append({
                'hash': row['file_hash'],
                'name': row['original_name'],
                'size': row['file_size'],
                'download_count': row['download_count'],
                'upload_time': row['upload_time']
            })
        
        conn.close()
        return jsonify({'success': True, 'files': files})

@app.route('/download/<file_hash>')
def download_file(file_hash):
    """Dosya indirme"""
    conn = get_db()
    file_record = conn.execute('SELECT * FROM files WHERE file_hash = ?', (file_hash,)).fetchone()
    
    if not file_record:
        conn.close()
        return "Dosya bulunamadı", 404
    
    file_path = file_record['file_path']
    original_name = file_record['original_name']
    
    if not os.path.exists(file_path):
        conn.close()
        return "Dosya disk üzerinde bulunamadı", 404
    
    # İndirme sayacını artır
    conn.execute('UPDATE files SET download_count = download_count + 1 WHERE file_hash = ?', (file_hash,))
    conn.commit()
    conn.close()
    
    return send_file(file_path, as_attachment=True, download_name=original_name)

# HTML Template
HTML_TEMPLATE = '''
<!DOCTYPE html>
<html lang="tr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🚀 BitSwapTorrent - GERÇEK Dosya Paylaşım</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { 
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            color: white;
            padding: 20px;
        }
        .container { max-width: 1000px; margin: 0 auto; }
        .header { 
            text-align: center; 
            margin-bottom: 40px; 
            animation: fadeInDown 1s ease;
        }
        .header h1 { 
            font-size: 3.5rem; 
            margin-bottom: 10px; 
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3); 
        }
        .version { 
            background: rgba(255,255,255,0.2); 
            padding: 8px 16px; 
            border-radius: 20px; 
            display: inline-block; 
            font-weight: bold;
        }
        
        .stats { 
            display: grid; 
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); 
            gap: 20px; 
            margin-bottom: 40px; 
        }
        .stat-card { 
            background: rgba(255,255,255,0.15); 
            backdrop-filter: blur(10px); 
            border-radius: 15px; 
            padding: 25px; 
            text-align: center; 
            transition: transform 0.3s ease;
        }
        .stat-card:hover { transform: translateY(-5px); }
        .stat-number { font-size: 2.5rem; font-weight: bold; margin-bottom: 8px; }
        .stat-label { opacity: 0.9; font-size: 0.9rem; }
        
        .upload-area { 
            background: rgba(255,255,255,0.1); 
            border: 3px dashed rgba(255,255,255,0.4); 
            border-radius: 20px; 
            padding: 50px; 
            text-align: center; 
            margin-bottom: 30px; 
            cursor: pointer; 
            transition: all 0.3s ease;
        }
        .upload-area:hover, .upload-area.drag-over { 
            background: rgba(255,255,255,0.2); 
            border-color: rgba(255,255,255,0.8); 
            transform: translateY(-3px);
        }
        .upload-icon { font-size: 4rem; margin-bottom: 15px; opacity: 0.8; }
        .upload-text { font-size: 1.3rem; margin-bottom: 8px; }
        .upload-subtext { opacity: 0.8; margin-bottom: 20px; }
        
        .btn { 
            background: linear-gradient(45deg, #ff6b6b, #ee5a24); 
            color: white; 
            border: none; 
            padding: 12px 24px; 
            border-radius: 25px; 
            font-size: 1rem; 
            font-weight: 600; 
            cursor: pointer; 
            transition: all 0.3s ease;
            text-decoration: none;
            display: inline-block;
        }
        .btn:hover { 
            transform: translateY(-2px); 
            box-shadow: 0 8px 16px rgba(0,0,0,0.2);
        }
        
        .files-section { 
            background: rgba(255,255,255,0.1); 
            border-radius: 20px; 
            padding: 30px; 
        }
        .files-header { 
            display: flex; 
            justify-content: space-between; 
            align-items: center; 
            margin-bottom: 20px; 
        }
        .file-item { 
            background: rgba(255,255,255,0.1); 
            border-radius: 12px; 
            padding: 20px; 
            margin-bottom: 15px; 
            display: flex; 
            justify-content: space-between; 
            align-items: center;
            transition: all 0.3s ease;
        }
        .file-item:hover { background: rgba(255,255,255,0.2); }
        .file-info h4 { margin-bottom: 5px; }
        .file-details { opacity: 0.8; font-size: 0.9rem; }
        
        #fileInput { display: none; }
        .progress { 
            width: 100%; 
            height: 12px; 
            background: rgba(255,255,255,0.2); 
            border-radius: 6px; 
            overflow: hidden; 
            margin: 15px 0;
        }
        .progress-bar { 
            height: 100%; 
            background: linear-gradient(45deg, #00b894, #00cec9); 
            width: 0%; 
            transition: width 0.3s ease;
        }
        
        .notification { 
            position: fixed; 
            top: 20px; 
            right: 20px; 
            background: rgba(0,0,0,0.8); 
            color: white; 
            padding: 15px 20px; 
            border-radius: 8px; 
            z-index: 1000;
            animation: slideIn 0.3s ease;
        }
        
        .hidden { display: none !important; }
        
        @keyframes fadeInDown {
            from { opacity: 0; transform: translateY(-30px); }
            to { opacity: 1; transform: translateY(0); }
        }
        
        @keyframes slideIn {
            from { transform: translateX(100%); opacity: 0; }
            to { transform: translateX(0); opacity: 1; }
        }
        
        @media (max-width: 768px) {
            .header h1 { font-size: 2.5rem; }
            .upload-area { padding: 30px 20px; }
            .stats { grid-template-columns: repeat(2, 1fr); }
            .files-header { flex-direction: column; gap: 10px; }
            .file-item { flex-direction: column; align-items: flex-start; gap: 10px; }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 BitSwapTorrent</h1>
            <div class="version">✨ GERÇEK Flask v1.0 ✨</div>
        </div>
        
        <div class="stats">
            <div class="stat-card">
                <div class="stat-number" id="totalFiles">0</div>
                <div class="stat-label">📁 Toplam Dosya</div>
            </div>
            <div class="stat-card">
                <div class="stat-number" id="totalSize">0 MB</div>
                <div class="stat-label">💾 Toplam Boyut</div>
            </div>
            <div class="stat-card">
                <div class="stat-number" id="totalDownloads">0</div>
                <div class="stat-label">⬇️ İndirme Sayısı</div>
            </div>
            <div class="stat-card">
                <div class="stat-number">🔥 CANLI</div>
                <div class="stat-label">🌐 Server Durumu</div>
            </div>
        </div>
        
        <div class="upload-area" onclick="selectFile()" ondrop="handleDrop(event)" ondragover="handleDragOver(event)">
            <div class="upload-icon">☁️</div>
            <div class="upload-text">Dosya Yükle ve Dünyayla Paylaş</div>
            <div class="upload-subtext">Her türlü dosyayı sürükleyip bırakın veya tıklayın</div>
            <button class="btn">📁 Dosya Seç</button>
        </div>
        
        <input type="file" id="fileInput" multiple onchange="uploadFiles(this.files)">
        
        <div id="uploadProgress" class="hidden">
            <h3>⚡ Yükleniyor...</h3>
            <div class="progress">
                <div class="progress-bar" id="progressBar"></div>
            </div>
            <p id="progressText">0%</p>
        </div>
        
        <div class="files-section">
            <div class="files-header">
                <h2>📚 Paylaşılan Dosyalar</h2>
                <button class="btn" onclick="loadFiles()">🔄 Yenile</button>
            </div>
            <div id="filesList">
                <div style="text-align: center; padding: 40px; opacity: 0.8;">
                    📥 Dosyalar yükleniyor...
                </div>
            </div>
        </div>
    </div>
    
    <script>
        // Sayfa yüklenince
        document.addEventListener('DOMContentLoaded', function() {
            loadStats();
            loadFiles();
            setInterval(loadStats, 3000);
        });
        
        function selectFile() {
            document.getElementById('fileInput').click();
        }
        
        function handleDrop(e) {
            e.preventDefault();
            e.target.classList.remove('drag-over');
            uploadFiles(e.dataTransfer.files);
        }
        
        function handleDragOver(e) {
            e.preventDefault();
            e.target.classList.add('drag-over');
        }
        
        function uploadFiles(files) {
            if (files.length === 0) return;
            
            const progress = document.getElementById('uploadProgress');
            const bar = document.getElementById('progressBar');
            const text = document.getElementById('progressText');
            
            progress.classList.remove('hidden');
            
            Array.from(files).forEach((file, index) => {
                const formData = new FormData();
                formData.append('file', file);
                
                const xhr = new XMLHttpRequest();
                
                xhr.upload.onprogress = function(e) {
                    if (e.lengthComputable) {
                        const percent = (e.loaded / e.total) * 100;
                        bar.style.width = percent + '%';
                        text.textContent = Math.round(percent) + '% - ' + file.name;
                    }
                };
                
                xhr.onload = function() {
                    if (xhr.status === 200) {
                        const response = JSON.parse(xhr.responseText);
                        if (response.success) {
                            showNotification('✅ ' + file.name + ' yüklendi!');
                            if (response.share_url) {
                                navigator.clipboard.writeText(response.share_url).catch(() => {});
                                showNotification('🔗 Link panoya kopyalandı!');
                            }
                            loadStats();
                            loadFiles();
                        } else {
                            showNotification('❌ Hata: ' + response.message);
                        }
                    }
                    
                    if (index === files.length - 1) {
                        progress.classList.add('hidden');
                    }
                };
                
                xhr.open('POST', '/api/upload');
                xhr.send(formData);
            });
        }
        
        function loadStats() {
            fetch('/api/files?action=stats')
                .then(r => r.json())
                .then(data => {
                    if (data.success) {
                        document.getElementById('totalFiles').textContent = data.stats.total_files;
                        document.getElementById('totalSize').textContent = formatBytes(data.stats.total_size);
                        document.getElementById('totalDownloads').textContent = data.stats.total_downloads;
                    }
                })
                .catch(() => {});
        }
        
        function loadFiles() {
            fetch('/api/files?action=list')
                .then(r => r.json())
                .then(data => {
                    const list = document.getElementById('filesList');
                    if (data.success && data.files.length > 0) {
                        list.innerHTML = data.files.map(file => `
                            <div class="file-item">
                                <div class="file-info">
                                    <h4>📄 ${file.name}</h4>
                                    <div class="file-details">
                                        💾 ${formatBytes(file.size)} • ⬇️ ${file.download_count} indirme
                                    </div>
                                </div>
                                <button class="btn" onclick="downloadFile('${file.hash}', '${file.name}')">
                                    📥 İndir
                                </button>
                            </div>
                        `).join('');
                    } else {
                        list.innerHTML = '<div style="text-align: center; padding: 40px; opacity: 0.8;">🎯 Henüz dosya yok. İlk dosyayı sen yükle!</div>';
                    }
                })
                .catch(() => {});
        }
        
        function downloadFile(hash, name) {
            const link = document.createElement('a');
            link.href = '/download/' + hash;
            link.download = name;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            
            showNotification('📥 ' + name + ' indiriliyor...');
            setTimeout(loadStats, 1000);
        }
        
        function formatBytes(bytes) {
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
        }
        
        function showNotification(message) {
            const notification = document.createElement('div');
            notification.className = 'notification';
            notification.textContent = message;
            document.body.appendChild(notification);
            setTimeout(() => notification.remove(), 3000);
        }
    </script>
</body>
</html>
'''

if __name__ == '__main__':
    init_db()
    print("""
🎉 BitSwapTorrent Flask Server Başlatılıyor!

📍 URL: http://localhost:5000
📁 Uploads: uploads/ klasörü
💾 Database: bitswap.db

✅ Gerçek dosya paylaşımı başladı!
🔗 Tarayıcıda http://localhost:5000 adresini açın

Durdurmak için Ctrl+C
""")
    
    try:
        app.run(host='0.0.0.0', port=5000, debug=False)
    except KeyboardInterrupt:
        print("\n🛑 Server durduruldu!")
