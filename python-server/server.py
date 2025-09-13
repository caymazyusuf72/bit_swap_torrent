#!/usr/bin/env python3
"""
BitSwapTorrent - Gerçek P2P Dosya Paylaşım Sistemi
Python HTTP Server Implementation
"""

import os
import json
import hashlib
import sqlite3
import mimetypes
import urllib.parse
from datetime import datetime
from http.server import HTTPServer, SimpleHTTPRequestHandler
from urllib.parse import urlparse, parse_qs
import cgi

class BitSwapHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        self.upload_dir = "uploads"
        self.db_path = "database.sqlite"
        os.makedirs(self.upload_dir, exist_ok=True)
        self.init_database()
        super().__init__(*args, **kwargs)
    
    def init_database(self):
        """Initialize SQLite database"""
        conn = sqlite3.connect(self.db_path)
        conn.execute('''
            CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_hash TEXT UNIQUE NOT NULL,
                original_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                mime_type TEXT,
                upload_time DATETIME DEFAULT CURRENT_TIMESTAMP,
                download_count INTEGER DEFAULT 0,
                uploader_ip TEXT
            )
        ''')
        conn.execute('''
            CREATE TABLE IF NOT EXISTS peers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_hash TEXT,
                peer_ip TEXT,
                last_seen DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        ''')
        conn.commit()
        conn.close()
    
    def do_GET(self):
        """Handle GET requests"""
        parsed_path = urlparse(self.path)
        path = parsed_path.path
        
        if path == '/':
            self.serve_main_page()
        elif path == '/api/files':
            self.handle_api_files(parsed_path)
        elif path.startswith('/api/download'):
            self.handle_download(parsed_path)
        elif path.startswith('/uploads/'):
            self.serve_upload_file()
        else:
            super().do_GET()
    
    def do_POST(self):
        """Handle POST requests"""
        parsed_path = urlparse(self.path)
        
        if parsed_path.path == '/api/upload':
            self.handle_upload()
        else:
            self.send_error(404)
    
    def serve_main_page(self):
        """Serve the main HTML page"""
        html = '''<!DOCTYPE html>
<html lang="tr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BitSwapTorrent - Gerçek P2P Dosya Paylaşım</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            color: white;
        }
        .container { max-width: 1200px; margin: 0 auto; padding: 2rem; }
        .header { text-align: center; margin-bottom: 3rem; }
        .header h1 { font-size: 3rem; margin-bottom: 1rem; text-shadow: 2px 2px 4px rgba(0,0,0,0.3); }
        .version { background: rgba(255,255,255,0.2); padding: 0.5rem 1rem; border-radius: 25px; display: inline-block; }
        
        .upload-zone {
            background: rgba(255,255,255,0.1);
            border: 3px dashed rgba(255,255,255,0.3);
            border-radius: 20px;
            padding: 3rem;
            text-align: center;
            margin-bottom: 2rem;
            cursor: pointer;
            transition: all 0.3s ease;
        }
        .upload-zone:hover, .upload-zone.drag-over {
            background: rgba(255,255,255,0.2);
            border-color: rgba(255,255,255,0.8);
            transform: translateY(-5px);
        }
        .upload-zone i { font-size: 4rem; margin-bottom: 1rem; opacity: 0.8; }
        .upload-zone h3 { font-size: 1.5rem; margin-bottom: 1rem; }
        .upload-zone p { opacity: 0.8; margin-bottom: 2rem; }
        
        .btn {
            background: linear-gradient(45deg, #ff6b6b, #ee5a24);
            color: white;
            border: none;
            padding: 1rem 2rem;
            border-radius: 50px;
            font-size: 1rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.3s ease;
            text-decoration: none;
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
        }
        .btn:hover { 
            transform: translateY(-2px);
            box-shadow: 0 10px 20px rgba(0,0,0,0.2);
        }
        
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1.5rem;
            margin: 2rem 0;
        }
        .stat-card {
            background: rgba(255,255,255,0.15);
            backdrop-filter: blur(10px);
            border-radius: 15px;
            padding: 2rem;
            text-align: center;
        }
        .stat-value { font-size: 2.5rem; font-weight: bold; margin-bottom: 0.5rem; }
        .stat-label { opacity: 0.8; }
        
        .file-list {
            background: rgba(255,255,255,0.1);
            backdrop-filter: blur(10px);
            border-radius: 20px;
            padding: 2rem;
            margin-top: 2rem;
        }
        .file-item {
            background: rgba(255,255,255,0.1);
            border-radius: 10px;
            padding: 1rem;
            margin-bottom: 1rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .file-info h4 { margin-bottom: 0.25rem; }
        .file-details { opacity: 0.8; font-size: 0.9rem; }
        
        #fileInput { display: none; }
        .progress { 
            width: 100%; 
            height: 10px; 
            background: rgba(255,255,255,0.2); 
            border-radius: 5px; 
            overflow: hidden; 
            margin: 1rem 0;
        }
        .progress-bar { 
            height: 100%; 
            background: linear-gradient(45deg, #00b894, #00cec9); 
            width: 0%; 
            transition: width 0.3s;
        }
        .hidden { display: none; }
        .notification {
            position: fixed;
            top: 20px;
            right: 20px;
            background: rgba(0,0,0,0.8);
            color: white;
            padding: 1rem 1.5rem;
            border-radius: 10px;
            z-index: 1000;
            animation: slideIn 0.3s ease;
        }
        
        @keyframes slideIn {
            from { transform: translateX(100%); opacity: 0; }
            to { transform: translateX(0); opacity: 1; }
        }
        
        @media (max-width: 768px) {
            .container { padding: 1rem; }
            .header h1 { font-size: 2rem; }
            .upload-zone { padding: 2rem 1rem; }
            .stats-grid { grid-template-columns: repeat(2, 1fr); }
        }
    </style>
    <link href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css" rel="stylesheet">
</head>
<body>
    <div class="container">
        <div class="header">
            <h1><i class="fas fa-exchange-alt"></i> BitSwapTorrent</h1>
            <div class="version">🚀 GERÇEK Python v1.0</div>
        </div>
        
        <div class="stats-grid" id="statsGrid">
            <div class="stat-card">
                <div class="stat-value" id="totalFiles">0</div>
                <div class="stat-label">📁 Toplam Dosya</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="totalSize">0 MB</div>
                <div class="stat-label">💾 Toplam Boyut</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="totalDownloads">0</div>
                <div class="stat-label">⬇️ İndirme</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">🔥 Aktif</div>
                <div class="stat-label">🌐 Durum</div>
            </div>
        </div>
        
        <div class="upload-zone" onclick="triggerFileUpload()" ondrop="handleDrop(event)" ondragover="handleDragOver(event)">
            <i class="fas fa-cloud-upload-alt"></i>
            <h3>Dosya Yükle ve Dünya ile Paylaş</h3>
            <p>Her türlü dosyayı sürükleyip bırakın veya seçin</p>
            <button class="btn">
                <i class="fas fa-plus"></i>
                Dosya Seç
            </button>
        </div>
        
        <input type="file" id="fileInput" multiple onchange="handleFileSelect(event)">
        
        <div id="uploadProgress" class="hidden">
            <h4>⚡ Yükleniyor...</h4>
            <div class="progress">
                <div class="progress-bar" id="progressBar"></div>
            </div>
            <p id="progressText">0%</p>
        </div>
        
        <div class="file-list">
            <h2>📚 Paylaşılan Dosyalar</h2>
            <button class="btn" onclick="loadFiles()">
                <i class="fas fa-refresh"></i>
                Yenile
            </button>
            <div id="filesList">
                <div style="text-align: center; padding: 2rem; opacity: 0.8;">
                    Dosyalar yükleniyor...
                </div>
            </div>
        </div>
    </div>
    
    <script>
        // Load data on page load
        document.addEventListener('DOMContentLoaded', function() {
            loadStats();
            loadFiles();
            setInterval(() => {
                loadStats();
                loadFiles();
            }, 5000);
        });
        
        function triggerFileUpload() {
            document.getElementById('fileInput').click();
        }
        
        function handleFileSelect(event) {
            uploadFiles(Array.from(event.target.files));
        }
        
        function handleDrop(event) {
            event.preventDefault();
            event.currentTarget.classList.remove('drag-over');
            uploadFiles(Array.from(event.dataTransfer.files));
        }
        
        function handleDragOver(event) {
            event.preventDefault();
            event.currentTarget.classList.add('drag-over');
        }
        
        function uploadFiles(files) {
            if (files.length === 0) return;
            
            const progressDiv = document.getElementById('uploadProgress');
            const progressBar = document.getElementById('progressBar');
            const progressText = document.getElementById('progressText');
            
            progressDiv.classList.remove('hidden');
            
            files.forEach((file, index) => {
                const formData = new FormData();
                formData.append('file', file);
                
                const xhr = new XMLHttpRequest();
                
                xhr.upload.addEventListener('progress', (e) => {
                    if (e.lengthComputable) {
                        const percentComplete = (e.loaded / e.total) * 100;
                        progressBar.style.width = percentComplete + '%';
                        progressText.textContent = Math.round(percentComplete) + '% - ' + file.name;
                    }
                });
                
                xhr.addEventListener('load', function() {
                    if (xhr.status === 200) {
                        const response = JSON.parse(xhr.responseText);
                        if (response.success) {
                            showNotification(`✅ ${file.name} başarıyla yüklendi!`);
                            if (response.magnet_url) {
                                navigator.clipboard.writeText(response.share_url).catch(() => {});
                                showNotification('🔗 İndirme linki panoya kopyalandı!');
                            }
                            loadStats();
                            loadFiles();
                        } else {
                            showNotification(`❌ Hata: ${response.message}`);
                        }
                    }
                    
                    if (index === files.length - 1) {
                        progressDiv.classList.add('hidden');
                        document.getElementById('fileInput').value = '';
                    }
                });
                
                xhr.open('POST', '/api/upload');
                xhr.send(formData);
            });
        }
        
        function loadStats() {
            fetch('/api/files?action=stats')
                .then(response => response.json())
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
                .then(response => response.json())
                .then(data => {
                    if (data.success) {
                        displayFiles(data.files);
                    }
                })
                .catch(() => {});
        }
        
        function displayFiles(files) {
            const filesList = document.getElementById('filesList');
            
            if (files.length === 0) {
                filesList.innerHTML = '<div style="text-align: center; padding: 2rem; opacity: 0.8;">🎯 Henüz dosya yok. İlk dosyayı sen yükle!</div>';
                return;
            }
            
            filesList.innerHTML = files.map(file => `
                <div class="file-item">
                    <div class="file-info">
                        <h4>📄 ${file.name}</h4>
                        <div class="file-details">
                            💾 ${formatBytes(file.size)} • ⬇️ ${file.download_count} indirme
                        </div>
                    </div>
                    <div>
                        <button class="btn" onclick="downloadFile('${file.hash}', '${file.name}')">
                            <i class="fas fa-download"></i>
                            İndir
                        </button>
                    </div>
                </div>
            `).join('');
        }
        
        function downloadFile(hash, name) {
            const link = document.createElement('a');
            link.href = `/api/download?hash=${hash}`;
            link.download = name;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            
            showNotification(`📥 ${name} indiriliyor...`);
            setTimeout(() => { loadStats(); loadFiles(); }, 1000);
        }
        
        function formatBytes(bytes) {
            const sizes = ['B', 'KB', 'MB', 'GB'];
            if (bytes === 0) return '0 B';
            const i = Math.floor(Math.log(bytes) / Math.log(1024));
            return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
        }
        
        function showNotification(message) {
            const notification = document.createElement('div');
            notification.className = 'notification';
            notification.textContent = message;
            document.body.appendChild(notification);
            
            setTimeout(() => {
                notification.style.animation = 'slideIn 0.3s ease reverse';
                setTimeout(() => notification.remove(), 300);
            }, 3000);
        }
    </script>
</body>
</html>'''
        
        self.send_response(200)
        self.send_header('Content-type', 'text/html; charset=utf-8')
        self.end_headers()
        self.wfile.write(html.encode('utf-8'))
    
    def handle_api_files(self, parsed_path):
        """Handle files API requests"""
        query = parse_qs(parsed_path.query)
        action = query.get('action', ['list'])[0]
        
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        
        if action == 'stats':
            cursor = conn.execute('SELECT COUNT(*) as total_files, SUM(file_size) as total_size, SUM(download_count) as total_downloads FROM files')
            row = cursor.fetchone()
            stats = {
                'total_files': row['total_files'] or 0,
                'total_size': row['total_size'] or 0,
                'total_downloads': row['total_downloads'] or 0
            }
            response = {'success': True, 'stats': stats}
        else:
            cursor = conn.execute('SELECT * FROM files ORDER BY upload_time DESC LIMIT 50')
            files = []
            for row in cursor.fetchall():
                files.append({
                    'hash': row['file_hash'],
                    'name': row['original_name'],
                    'size': row['file_size'],
                    'download_count': row['download_count'],
                    'upload_time': row['upload_time']
                })
            response = {'success': True, 'files': files}
        
        conn.close()
        
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(json.dumps(response).encode('utf-8'))
    
    def handle_upload(self):
        """Handle file upload"""
        try:
            content_type = self.headers['Content-Type']
            if not content_type.startswith('multipart/form-data'):
                raise ValueError("Invalid content type")
            
            # Parse multipart form data
            form = cgi.FieldStorage(
                fp=self.rfile,
                headers=self.headers,
                environ={'REQUEST_METHOD': 'POST'}
            )
            
            if 'file' not in form:
                raise ValueError("No file uploaded")
            
            file_item = form['file']
            if not file_item.filename:
                raise ValueError("No file selected")
            
            # Read file content
            file_content = file_item.file.read()
            original_name = file_item.filename
            file_size = len(file_content)
            
            # Generate hash
            file_hash = hashlib.sha256(file_content).hexdigest()
            
            # Check if file exists
            conn = sqlite3.connect(self.db_path)
            cursor = conn.execute('SELECT * FROM files WHERE file_hash = ?', (file_hash,))
            existing = cursor.fetchone()
            
            if existing:
                conn.close()
                response = {
                    'success': True,
                    'message': 'File already exists',
                    'hash': file_hash,
                    'existing': True,
                    'share_url': f"http://{self.headers['Host']}/api/download?hash={file_hash}"
                }
            else:
                # Save file
                filename = f"{file_hash}_{original_name}"
                file_path = os.path.join(self.upload_dir, filename)
                
                with open(file_path, 'wb') as f:
                    f.write(file_content)
                
                # Get MIME type
                mime_type = mimetypes.guess_type(original_name)[0] or 'application/octet-stream'
                
                # Save to database
                client_ip = self.client_address[0]
                conn.execute('''
                    INSERT INTO files (file_hash, original_name, file_path, file_size, mime_type, uploader_ip)
                    VALUES (?, ?, ?, ?, ?, ?)
                ''', (file_hash, original_name, file_path, file_size, mime_type, client_ip))
                conn.commit()
                conn.close()
                
                response = {
                    'success': True,
                    'message': 'File uploaded successfully',
                    'hash': file_hash,
                    'original_name': original_name,
                    'size': file_size,
                    'magnet_url': f"magnet:?xt=urn:sha256:{file_hash}&dn={urllib.parse.quote(original_name)}&xl={file_size}",
                    'share_url': f"http://{self.headers['Host']}/api/download?hash={file_hash}"
                }
        
        except Exception as e:
            response = {'success': False, 'message': str(e)}
        
        self.send_response(200 if response['success'] else 400)
        self.send_header('Content-type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(json.dumps(response).encode('utf-8'))
    
    def handle_download(self, parsed_path):
        """Handle file download"""
        query = parse_qs(parsed_path.query)
        file_hash = query.get('hash', [None])[0]
        
        if not file_hash:
            self.send_error(400, "Missing hash parameter")
            return
        
        conn = sqlite3.connect(self.db_path)
        cursor = conn.execute('SELECT * FROM files WHERE file_hash = ?', (file_hash,))
        file_record = cursor.fetchone()
        
        if not file_record:
            conn.close()
            self.send_error(404, "File not found")
            return
        
        file_path = file_record[3]  # file_path column
        original_name = file_record[2]  # original_name column
        file_size = file_record[4]  # file_size column
        mime_type = file_record[5]  # mime_type column
        
        if not os.path.exists(file_path):
            conn.close()
            self.send_error(404, "File not found on disk")
            return
        
        # Update download count
        conn.execute('UPDATE files SET download_count = download_count + 1 WHERE file_hash = ?', (file_hash,))
        conn.commit()
        conn.close()
        
        # Send file
        self.send_response(200)
        self.send_header('Content-Type', mime_type or 'application/octet-stream')
        self.send_header('Content-Disposition', f'attachment; filename="{original_name}"')
        self.send_header('Content-Length', str(file_size))
        self.end_headers()
        
        with open(file_path, 'rb') as f:
            while True:
                chunk = f.read(8192)
                if not chunk:
                    break
                self.wfile.write(chunk)

def run_server(port=8080):
    """Run the BitSwapTorrent server"""
    server_address = ('', port)
    httpd = HTTPServer(server_address, BitSwapHandler)
    print(f"""
🚀 BitSwapTorrent Server Başlatıldı!

📍 Adres: http://localhost:{port}
📁 Upload klasörü: uploads/
💾 Database: database.sqlite

✅ Artık dosya yükleyip paylaşabilirsin!
🔗 Tarayıcıda http://localhost:{port} adresini aç

Durdurmak için Ctrl+C
""")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n🛑 Server durduruldu!")
        httpd.server_close()

if __name__ == '__main__':
    run_server()
