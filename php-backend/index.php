<?php
// BitSwapTorrent - Real P2P File Sharing System
?>
<!DOCTYPE html>
<html lang="tr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BitSwapTorrent - Gerçek P2P Dosya Paylaşım Sistemi</title>
    <link href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css" rel="stylesheet">
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <style>
        /* Modern Dark Theme */
        :root {
            --primary: #3b82f6;
            --primary-dark: #2563eb;
            --secondary: #6b7280;
            --success: #10b981;
            --danger: #ef4444;
            --warning: #f59e0b;
            --info: #06b6d4;
            
            --bg-primary: #0f172a;
            --bg-secondary: #1e293b;
            --bg-tertiary: #334155;
            --bg-card: #1e293b;
            --bg-modal: rgba(15, 23, 42, 0.9);
            
            --text-primary: #f8fafc;
            --text-secondary: #cbd5e1;
            --text-muted: #94a3b8;
            
            --border: #334155;
            --border-light: #475569;
            
            --radius: 0.5rem;
            --radius-lg: 0.75rem;
            --radius-xl: 1rem;
            
            --transition: all 0.2s ease;
        }

        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.5;
        }

        .container {
            display: grid;
            grid-template-columns: 250px 1fr;
            grid-template-rows: auto 1fr;
            grid-template-areas: 
                "header header"
                "nav main";
            min-height: 100vh;
        }

        /* Header */
        .header {
            grid-area: header;
            background: var(--bg-secondary);
            border-bottom: 1px solid var(--border);
            padding: 1rem 2rem;
        }

        .header-content {
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .logo {
            display: flex;
            align-items: center;
            gap: 0.75rem;
        }

        .logo i {
            font-size: 1.5rem;
            color: var(--primary);
        }

        .logo h1 {
            font-size: 1.25rem;
            font-weight: 600;
        }

        .version {
            background: var(--bg-tertiary);
            color: var(--text-secondary);
            padding: 0.25rem 0.5rem;
            border-radius: var(--radius);
            font-size: 0.75rem;
            font-weight: 500;
        }

        .header-actions {
            display: flex;
            gap: 0.75rem;
        }

        /* Navigation */
        .nav {
            grid-area: nav;
            background: var(--bg-secondary);
            padding: 1.5rem 1rem;
            border-right: 1px solid var(--border);
            overflow-y: auto;
        }

        .nav-item {
            display: flex;
            align-items: center;
            gap: 0.75rem;
            padding: 0.75rem 1rem;
            border-radius: var(--radius);
            cursor: pointer;
            transition: var(--transition);
            color: var(--text-secondary);
            margin-bottom: 0.5rem;
            position: relative;
        }

        .nav-item:hover {
            background: var(--bg-tertiary);
            color: var(--text-primary);
        }

        .nav-item.active {
            background: var(--primary);
            color: white;
        }

        /* Main Content */
        .main {
            grid-area: main;
            padding: 2rem;
            overflow-y: auto;
        }

        .tab-content {
            display: none;
        }

        .tab-content.active {
            display: block;
        }

        /* Upload Area */
        .upload-section {
            background: var(--bg-card);
            border: 2px dashed var(--border);
            border-radius: var(--radius-lg);
            padding: 2rem;
            text-align: center;
            margin-bottom: 2rem;
            transition: var(--transition);
            cursor: pointer;
        }

        .upload-section:hover,
        .upload-section.drag-over {
            border-color: var(--primary);
            background: rgba(59, 130, 246, 0.05);
        }

        .upload-section i {
            font-size: 3rem;
            color: var(--text-muted);
            margin-bottom: 1rem;
        }

        .upload-section h3 {
            margin-bottom: 0.5rem;
            color: var(--text-primary);
        }

        .upload-section p {
            color: var(--text-muted);
            margin-bottom: 1.5rem;
        }

        /* File List */
        .file-list {
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: var(--radius-lg);
            overflow: hidden;
        }

        .file-header {
            padding: 1.5rem;
            border-bottom: 1px solid var(--border);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .file-item {
            padding: 1.5rem;
            border-bottom: 1px solid var(--border);
            display: flex;
            align-items: center;
            gap: 1rem;
            transition: var(--transition);
        }

        .file-item:hover {
            background: var(--bg-tertiary);
        }

        .file-item:last-child {
            border-bottom: none;
        }

        .file-icon {
            width: 3rem;
            height: 3rem;
            background: var(--bg-tertiary);
            border-radius: var(--radius);
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 1.25rem;
            color: var(--text-secondary);
        }

        .file-info {
            flex: 1;
        }

        .file-name {
            font-weight: 500;
            margin-bottom: 0.25rem;
            color: var(--text-primary);
        }

        .file-details {
            font-size: 0.875rem;
            color: var(--text-muted);
        }

        .file-actions {
            display: flex;
            gap: 0.5rem;
        }

        /* Buttons */
        .btn {
            padding: 0.75rem 1.5rem;
            border: none;
            border-radius: var(--radius);
            font-size: 0.875rem;
            font-weight: 500;
            cursor: pointer;
            transition: var(--transition);
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            text-decoration: none;
        }

        .btn-primary {
            background: var(--primary);
            color: white;
        }

        .btn-primary:hover {
            background: var(--primary-dark);
        }

        .btn-secondary {
            background: var(--bg-tertiary);
            color: var(--text-secondary);
            border: 1px solid var(--border);
        }

        .btn-secondary:hover {
            background: var(--bg-card);
            color: var(--text-primary);
        }

        .btn-small {
            padding: 0.5rem 1rem;
            font-size: 0.75rem;
        }

        /* Stats */
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1.5rem;
            margin-bottom: 2rem;
        }

        .stat-card {
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: var(--radius-lg);
            padding: 1.5rem;
            text-align: center;
        }

        .stat-value {
            font-size: 2rem;
            font-weight: 700;
            color: var(--primary);
            margin-bottom: 0.5rem;
        }

        .stat-label {
            color: var(--text-muted);
            font-size: 0.875rem;
        }

        /* Loading */
        .loading {
            text-align: center;
            padding: 2rem;
            color: var(--text-muted);
        }

        /* Progress */
        .progress {
            width: 100%;
            height: 4px;
            background: var(--bg-tertiary);
            border-radius: 2px;
            overflow: hidden;
            margin: 1rem 0;
        }

        .progress-bar {
            height: 100%;
            background: var(--primary);
            transition: width 0.3s ease;
        }

        /* Responsive */
        @media (max-width: 768px) {
            .container {
                grid-template-columns: 1fr;
                grid-template-areas: 
                    "header"
                    "main";
            }
            
            .nav {
                display: none;
            }
            
            .main {
                padding: 1rem;
            }
            
            .stats-grid {
                grid-template-columns: repeat(2, 1fr);
            }
        }

        #fileInput {
            display: none;
        }

        /* Utility */
        .text-success { color: var(--success); }
        .text-danger { color: var(--danger); }
        .text-warning { color: var(--warning); }
        .text-info { color: var(--info); }
        
        .mb-2 { margin-bottom: 1rem; }
        .mt-2 { margin-top: 1rem; }
        .hidden { display: none; }
    </style>
</head>
<body>
    <div class="container">
        <!-- Header -->
        <header class="header">
            <div class="header-content">
                <div class="logo">
                    <i class="fas fa-exchange-alt"></i>
                    <h1>BitSwapTorrent</h1>
                    <span class="version">GERÇEK v1.0</span>
                </div>
                <div class="header-actions">
                    <button class="btn btn-primary" onclick="triggerFileUpload()">
                        <i class="fas fa-plus"></i>
                        Dosya Yükle
                    </button>
                </div>
            </div>
        </header>

        <!-- Navigation -->
        <nav class="nav">
            <div class="nav-item active" data-tab="dashboard">
                <i class="fas fa-tachometer-alt"></i>
                <span>Dashboard</span>
            </div>
            <div class="nav-item" data-tab="files">
                <i class="fas fa-folder"></i>
                <span>Dosyalar</span>
            </div>
        </nav>

        <!-- Main Content -->
        <main class="main">
            <!-- Dashboard Tab -->
            <div class="tab-content active" id="dashboard">
                <!-- Stats -->
                <div class="stats-grid" id="statsGrid">
                    <div class="stat-card">
                        <div class="stat-value" id="totalFiles">0</div>
                        <div class="stat-label">Toplam Dosya</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value" id="totalSize">0 MB</div>
                        <div class="stat-label">Toplam Boyut</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value" id="activePeers">0</div>
                        <div class="stat-label">Aktif Peer</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value" id="totalDownloads">0</div>
                        <div class="stat-label">İndirme Sayısı</div>
                    </div>
                </div>

                <!-- Upload Section -->
                <div class="upload-section" onclick="triggerFileUpload()" ondrop="handleDrop(event)" ondragover="handleDragOver(event)">
                    <i class="fas fa-cloud-upload-alt"></i>
                    <h3>Dosya Yükle ve Paylaş</h3>
                    <p>Her türlü dosyayı yükleyebilir ve dünya genelinde paylaşabilirsiniz</p>
                    <button class="btn btn-primary">Dosya Seç</button>
                </div>

                <input type="file" id="fileInput" multiple onchange="handleFileSelect(event)">

                <!-- Progress -->
                <div id="uploadProgress" class="hidden">
                    <h4>Yükleniyor...</h4>
                    <div class="progress">
                        <div class="progress-bar" id="progressBar"></div>
                    </div>
                    <p id="progressText">0%</p>
                </div>
            </div>

            <!-- Files Tab -->
            <div class="tab-content" id="files">
                <div class="file-list">
                    <div class="file-header">
                        <h2>Paylaşılan Dosyalar</h2>
                        <button class="btn btn-secondary btn-small" onclick="refreshFiles()">
                            <i class="fas fa-refresh"></i>
                            Yenile
                        </button>
                    </div>
                    <div id="filesList">
                        <div class="loading">Dosyalar yükleniyor...</div>
                    </div>
                </div>
            </div>
        </main>
    </div>

    <script>
        // Global variables
        let currentTab = 'dashboard';

        // Initialize app
        document.addEventListener('DOMContentLoaded', function() {
            loadStats();
            loadFiles();
            setupNavigation();
            
            // Refresh data every 10 seconds
            setInterval(() => {
                loadStats();
                if (currentTab === 'files') {
                    loadFiles();
                }
            }, 10000);
        });

        // Navigation
        function setupNavigation() {
            document.querySelectorAll('.nav-item').forEach(item => {
                item.addEventListener('click', (e) => {
                    const tab = e.currentTarget.dataset.tab;
                    if (tab) switchTab(tab);
                });
            });
        }

        function switchTab(tabName) {
            // Update nav items
            document.querySelectorAll('.nav-item').forEach(item => {
                item.classList.remove('active');
            });
            document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');
            
            // Update tab content
            document.querySelectorAll('.tab-content').forEach(content => {
                content.classList.remove('active');
            });
            document.getElementById(tabName).classList.add('active');
            
            currentTab = tabName;
            
            if (tabName === 'files') {
                loadFiles();
            }
        }

        // File operations
        function triggerFileUpload() {
            document.getElementById('fileInput').click();
        }

        function handleFileSelect(event) {
            const files = Array.from(event.target.files);
            uploadFiles(files);
        }

        function handleDrop(event) {
            event.preventDefault();
            event.currentTarget.classList.remove('drag-over');
            
            const files = Array.from(event.dataTransfer.files);
            uploadFiles(files);
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
                            showNotification(`${file.name} başarıyla yüklendi!`, 'success');
                            if (response.magnet_url) {
                                copyToClipboard(response.magnet_url);
                                showNotification('Magnet linki panoya kopyalandı!', 'info');
                            }
                            loadStats();
                            if (currentTab === 'files') {
                                loadFiles();
                            }
                        } else {
                            showNotification(`Hata: ${response.message}`, 'error');
                        }
                    } else {
                        showNotification(`Yükleme hatası: ${file.name}`, 'error');
                    }
                    
                    if (index === files.length - 1) {
                        progressDiv.classList.add('hidden');
                        document.getElementById('fileInput').value = '';
                    }
                });
                
                xhr.addEventListener('error', function() {
                    showNotification(`Ağ hatası: ${file.name}`, 'error');
                });
                
                xhr.open('POST', 'api/upload.php');
                xhr.send(formData);
            });
        }

        function loadStats() {
            fetch('api/files.php?action=stats')
                .then(response => response.json())
                .then(data => {
                    if (data.success) {
                        const stats = data.stats;
                        document.getElementById('totalFiles').textContent = stats.total_files;
                        document.getElementById('totalSize').textContent = formatBytes(stats.total_size);
                        document.getElementById('activePeers').textContent = stats.active_peers;
                        document.getElementById('totalDownloads').textContent = stats.total_downloads;
                    }
                })
                .catch(error => {
                    console.error('Stats yükleme hatası:', error);
                });
        }

        function loadFiles() {
            const filesList = document.getElementById('filesList');
            filesList.innerHTML = '<div class="loading">Dosyalar yükleniyor...</div>';
            
            fetch('api/files.php?action=list')
                .then(response => response.json())
                .then(data => {
                    if (data.success) {
                        displayFiles(data.files);
                    } else {
                        filesList.innerHTML = '<div class="loading">Dosya yüklenemedi: ' + data.error + '</div>';
                    }
                })
                .catch(error => {
                    filesList.innerHTML = '<div class="loading">Ağ hatası!</div>';
                    console.error('Dosya yükleme hatası:', error);
                });
        }

        function displayFiles(files) {
            const filesList = document.getElementById('filesList');
            
            if (files.length === 0) {
                filesList.innerHTML = '<div class="loading">Henüz dosya yok. İlk dosyayı sen yükle!</div>';
                return;
            }
            
            filesList.innerHTML = files.map(file => `
                <div class="file-item">
                    <div class="file-icon">
                        <i class="${getFileIcon(file.name)}"></i>
                    </div>
                    <div class="file-info">
                        <div class="file-name">${file.name}</div>
                        <div class="file-details">
                            ${formatBytes(file.size)} • ${file.download_count} indirme • ${file.peer_count} peer
                        </div>
                    </div>
                    <div class="file-actions">
                        <button class="btn btn-primary btn-small" onclick="downloadFile('${file.hash}', '${file.name}')">
                            <i class="fas fa-download"></i>
                            İndir
                        </button>
                        <button class="btn btn-secondary btn-small" onclick="copyMagnet('${file.magnet_url}')">
                            <i class="fas fa-link"></i>
                            Magnet
                        </button>
                    </div>
                </div>
            `).join('');
        }

        function refreshFiles() {
            loadFiles();
        }

        function downloadFile(hash, name) {
            const link = document.createElement('a');
            link.href = `api/download.php?hash=${hash}`;
            link.download = name;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            
            showNotification(`${name} indiriliyor...`, 'info');
            
            // Refresh stats after download
            setTimeout(() => {
                loadStats();
            }, 1000);
        }

        function copyMagnet(magnetUrl) {
            copyToClipboard(magnetUrl);
            showNotification('Magnet linki panoya kopyalandı!', 'success');
        }

        // Utility functions
        function formatBytes(bytes) {
            const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
            if (bytes === 0) return '0 B';
            const i = Math.floor(Math.log(bytes) / Math.log(1024));
            return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
        }

        function getFileIcon(filename) {
            const ext = filename.split('.').pop().toLowerCase();
            const iconMap = {
                'txt': 'fas fa-file-text',
                'pdf': 'fas fa-file-pdf',
                'doc': 'fas fa-file-word',
                'docx': 'fas fa-file-word',
                'xls': 'fas fa-file-excel',
                'xlsx': 'fas fa-file-excel',
                'zip': 'fas fa-file-archive',
                'rar': 'fas fa-file-archive',
                '7z': 'fas fa-file-archive',
                'iso': 'fas fa-file-archive',
                'mp3': 'fas fa-music',
                'wav': 'fas fa-music',
                'mp4': 'fas fa-film',
                'avi': 'fas fa-film',
                'mkv': 'fas fa-film',
                'jpg': 'fas fa-image',
                'png': 'fas fa-image',
                'gif': 'fas fa-image'
            };
            return iconMap[ext] || 'fas fa-file';
        }

        function copyToClipboard(text) {
            navigator.clipboard.writeText(text).catch(err => {
                // Fallback for older browsers
                const textArea = document.createElement('textarea');
                textArea.value = text;
                document.body.appendChild(textArea);
                textArea.select();
                document.execCommand('copy');
                document.body.removeChild(textArea);
            });
        }

        function showNotification(message, type = 'info') {
            const notification = document.createElement('div');
            notification.style.cssText = `
                position: fixed;
                top: 20px;
                right: 20px;
                padding: 1rem 1.5rem;
                background: var(--bg-secondary);
                border: 1px solid var(--border);
                border-radius: var(--radius-lg);
                color: var(--text-primary);
                z-index: 1001;
                animation: slideIn 0.3s ease;
                max-width: 400px;
                box-shadow: 0 10px 25px rgba(0,0,0,0.2);
            `;
            
            const colors = {
                success: 'var(--success)',
                error: 'var(--danger)',
                warning: 'var(--warning)',
                info: 'var(--info)'
            };
            
            const icons = {
                success: 'fa-check-circle',
                error: 'fa-times-circle',
                warning: 'fa-exclamation-triangle',
                info: 'fa-info-circle'
            };
            
            notification.innerHTML = `
                <i class="fas ${icons[type] || icons.info}" style="color: ${colors[type] || colors.info}; margin-right: 0.5rem;"></i>
                ${message}
            `;
            
            document.body.appendChild(notification);
            
            setTimeout(() => {
                notification.style.animation = 'slideOut 0.3s ease';
                setTimeout(() => notification.remove(), 300);
            }, 4000);
        }

        // Add CSS animations
        const style = document.createElement('style');
        style.textContent = `
            @keyframes slideIn {
                from { transform: translateX(100%); opacity: 0; }
                to { transform: translateX(0); opacity: 1; }
            }
            
            @keyframes slideOut {
                from { transform: translateX(0); opacity: 1; }
                to { transform: translateX(100%); opacity: 0; }
            }
        `;
        document.head.appendChild(style);
    </script>
</body>
</html>
