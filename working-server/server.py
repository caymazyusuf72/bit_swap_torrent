#!/usr/bin/env python3
"""
BitSwapTorrent - %100 Çalışan Dosya Paylaşım Sistemi
Sadece Python standard library kullanır - KURULUM GEREKTIRMEZ!
"""

import os
import sys
import json
import hashlib
import sqlite3
import mimetypes
import urllib.parse
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs
from datetime import datetime
import tempfile
import shutil

class BitSwapHandler(BaseHTTPRequestHandler):
    
    def __init__(self, *args, **kwargs):
        # Klasörleri oluştur
        os.makedirs("uploads", exist_ok=True)
        self.init_database()
        super().__init__(*args, **kwargs)
    
    def init_database(self):
        """SQLite database başlat"""
        conn = sqlite3.connect("bitswap.db")
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
    
    def log_message(self, format, *args):
        """Log mesajlarını sustur"""
        pass
    
    def do_GET(self):
        """GET istekleri"""
        parsed = urlparse(self.path)
        path = parsed.path
        
        if path == '/':
            self.serve_main_page()
        elif path == '/api/files':
            self.handle_api_files(parsed)
        elif path.startswith('/download/'):
            file_hash = path.split('/')[-1]
            self.handle_download(file_hash)
        else:
            self.send_error(404, "Sayfa bulunamadı")
    
    def do_POST(self):
        """POST istekleri"""
        parsed = urlparse(self.path)
        
        if parsed.path == '/api/upload':
            self.handle_upload()
        else:
            self.send_error(404, "Endpoint bulunamadı")
    
    def serve_main_page(self):
        """Ana sayfa HTML"""
        html = '''<!DOCTYPE html>
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
        .header { text-align: center; margin-bottom: 40px; }
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
            display: none;
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
            <div class="version">✨ SIFIR KURULUM Python v1.0 ✨</div>
        </div>
        
        <div class="stats">
            <div class="stat-card">
                <div class="stat-number" id="totalFiles">0</div>
                <div class="stat-label">📁 Dosya</div>
            </div>
            <div class="stat-card">
                <div class="stat-number" id="totalSize">0 MB</div>
                <div class="stat-label">💾 Boyut</div>
            </div>
            <div class="stat-card">
                <div class="stat-number" id="totalDownloads">0</div>
                <div class="stat-label">⬇️ İndirme</div>
            </div>
            <div class="stat-card">
                <div class="stat-number">🔥 CANLI</div>
                <div class="stat-label">🌐 Durum</div>
            </div>
        </div>
        
        <div class="upload-area" onclick="selectFile()" ondrop="handleDrop(event)" ondragover="handleDragOver(event)">
            <div class="upload-icon">☁️</div>
            <div class="upload-text">Dosya Yükle ve Paylaş</div>
            <div class="upload-subtext">Her türlü dosyayı sürükleyip bırakın veya tıklayın</div>
            <button class="btn">📁 Dosya Seç</button>
        </div>
        
        <input type="file" id="fileInput" multiple onchange="uploadFiles(this.files)">
        
        <div class="progress" id="uploadProgress">
            <div class="progress-bar" id="progressBar"></div>
        </div>
        <p id="progressText" style="text-align: center; margin: 10px 0;"></p>
        
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
        // Sayfa yüklendiğinde
        document.addEventListener('DOMContentLoaded', function() {
            loadStats();
            loadFiles();
            setInterval(() => {
                loadStats();
                loadFiles();
            }, 5000);
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
            
            progress.style.display = 'block';
            
            Array.from(files).forEach((file, index) => {
                const formData = new FormData();
                formData.append('file', file);
                
                const xhr = new XMLHttpRequest();
                
                xhr.upload.onprogress = function(e) {
                    if (e.lengthComputable) {
                        const percent = (e.loaded / e.total) * 100;
                        bar.style.width = percent + '%';
                        text.textContent = '⚡ ' + Math.round(percent) + '% - ' + file.name;
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
                    } else {
                        showNotification('❌ Yükleme hatası!');
                    }
                    
                    if (index === files.length - 1) {
                        progress.style.display = 'none';
