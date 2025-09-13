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
