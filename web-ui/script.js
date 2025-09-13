// BitSwapTorrent Web UI JavaScript
class BitSwapUI {
    constructor() {
        this.activeTab = 'dashboard';
        this.torrents = this.getMockTorrents();
        this.peers = this.getMockPeers();
        this.stats = this.getMockStats();
        
        this.init();
    }
    
    init() {
        this.setupEventListeners();
        this.startStatsUpdater();
        this.renderDashboard();
    }
    
    setupEventListeners() {
        // Tab switching
        document.querySelectorAll('.nav-item').forEach(item => {
            item.addEventListener('click', (e) => {
                const tab = e.currentTarget.dataset.tab;
                if (tab) this.switchTab(tab);
            });
        });
        
        // Filter buttons
        document.querySelectorAll('.filter-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                this.setActiveFilter(e.target);
                const filter = e.target.dataset.filter;
                this.filterTorrents(filter);
            });
        });
        
        // Torrent actions
        document.addEventListener('click', (e) => {
            if (e.target.closest('.action-btn')) {
                const actionBtn = e.target.closest('.action-btn');
                const action = this.getActionFromIcon(actionBtn);
                const torrentElement = actionBtn.closest('.torrent-item');
                this.handleTorrentAction(action, torrentElement);
            }
        });
        
        // Settings save
        document.addEventListener('click', (e) => {
            if (e.target.textContent === 'Kaydet') {
                this.saveSettings();
            }
        });
        
        // Modal events
        document.addEventListener('click', (e) => {
            if (e.target.classList.contains('modal')) {
                this.closeAllModals();
            }
        });
        
        // Escape key to close modals
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                this.closeAllModals();
            }
        });
    }
    
    switchTab(tabName) {
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
        
        this.activeTab = tabName;
        this.updateTabContent(tabName);
    }
    
    updateTabContent(tabName) {
        switch(tabName) {
            case 'torrents':
                this.renderTorrents();
                break;
            case 'peers':
                this.renderPeers();
                break;
            case 'dashboard':
                this.renderDashboard();
                break;
        }
    }
    
    renderTorrents() {
        const torrentList = document.querySelector('.torrent-list');
        torrentList.innerHTML = '';
        
        this.torrents.forEach(torrent => {
            const torrentElement = this.createTorrentElement(torrent);
            torrentList.appendChild(torrentElement);
        });
    }
    
    createTorrentElement(torrent) {
        const div = document.createElement('div');
        div.className = 'torrent-item';
        div.innerHTML = `
            <div class="torrent-icon">
                <i class="${this.getFileIcon(torrent.name)}"></i>
            </div>
            <div class="torrent-info">
                <div class="torrent-name">${torrent.name}</div>
                <div class="torrent-details">
                    <span class="size">${this.formatBytes(torrent.size)}</span>
                    <span class="status ${torrent.status.toLowerCase()}">${this.getStatusText(torrent.status)}</span>
                    <span class="peers">${torrent.peers} peers</span>
                </div>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: ${torrent.progress}%"></div>
                </div>
            </div>
            <div class="torrent-actions">
                <button class="action-btn" title="${torrent.status === 'downloading' ? 'Duraklat' : 'Başlat'}">
                    <i class="fas fa-${torrent.status === 'downloading' ? 'pause' : 'play'}"></i>
                </button>
                <button class="action-btn" title="Sil">
                    <i class="fas fa-trash"></i>
                </button>
            </div>
        `;
        return div;
    }
    
    renderPeers() {
        const peerList = document.querySelector('.peer-list');
        peerList.innerHTML = '';
        
        this.peers.forEach(peer => {
            const peerElement = this.createPeerElement(peer);
            peerList.appendChild(peerElement);
        });
    }
    
    createPeerElement(peer) {
        const div = document.createElement('div');
        div.className = 'peer-item';
        div.innerHTML = `
            <div class="peer-info">
                <div class="peer-id">${peer.ip}:${peer.port}</div>
                <div class="peer-details">
                    <span class="flag">${peer.flag}</span>
                    <span class="client">${peer.client}</span>
                </div>
            </div>
            <div class="peer-stats">
                <div class="stat">
                    <i class="fas fa-download text-success"></i>
                    <span>${this.formatSpeed(peer.downloadSpeed)}</span>
                </div>
                <div class="stat">
                    <i class="fas fa-upload text-info"></i>
                    <span>${this.formatSpeed(peer.uploadSpeed)}</span>
                </div>
            </div>
        `;
        return div;
    }
    
    renderDashboard() {
        this.updateStatsCards();
        this.updateActivityList();
    }
    
    updateStatsCards() {
        document.querySelector('.stat-card .stat-value').textContent = this.formatSpeed(this.stats.downloadSpeed);
        document.querySelectorAll('.stat-card .stat-value')[1].textContent = this.formatSpeed(this.stats.uploadSpeed);
        document.querySelectorAll('.stat-card .stat-value')[2].textContent = this.stats.connectedPeers;
        document.querySelectorAll('.stat-card .stat-value')[3].textContent = this.stats.activeTorrents;
    }
    
    updateActivityList() {
        // Activity list is already static in HTML, could be made dynamic here
    }
    
    setActiveFilter(button) {
        document.querySelectorAll('.filter-btn').forEach(btn => {
            btn.classList.remove('active');
        });
        button.classList.add('active');
    }
    
    filterTorrents(filter) {
        const filteredTorrents = filter === 'all' 
            ? this.torrents 
            : this.torrents.filter(torrent => torrent.status.toLowerCase() === filter);
        
        // Re-render with filtered torrents
        const torrentList = document.querySelector('.torrent-list');
        torrentList.innerHTML = '';
        
        filteredTorrents.forEach(torrent => {
            const torrentElement = this.createTorrentElement(torrent);
            torrentList.appendChild(torrentElement);
        });
    }
    
    handleTorrentAction(action, torrentElement) {
        const torrentName = torrentElement.querySelector('.torrent-name').textContent;
        const torrent = this.torrents.find(t => t.name === torrentName);
        
        if (!torrent) return;
        
        switch(action) {
            case 'pause':
                torrent.status = 'paused';
                this.showNotification(`${torrent.name} duraklatıldı`, 'info');
                break;
            case 'play':
                torrent.status = 'downloading';
                this.showNotification(`${torrent.name} başlatıldı`, 'success');
                break;
            case 'delete':
                if (confirm(`${torrent.name} torrent'ini silmek istediğiniz emin misiniz?`)) {
                    this.torrents = this.torrents.filter(t => t.name !== torrent.name);
                    this.showNotification(`${torrent.name} silindi`, 'warning');
                }
                break;
        }
        
        this.renderTorrents();
        this.updateStatsCards();
    }
    
    getActionFromIcon(actionBtn) {
        const icon = actionBtn.querySelector('i');
        if (icon.classList.contains('fa-pause')) return 'pause';
        if (icon.classList.contains('fa-play')) return 'play';
        if (icon.classList.contains('fa-trash')) return 'delete';
        return 'unknown';
    }
    
    saveSettings() {
        // Collect settings from form
        const settings = {
            downloadDir: document.querySelector('input[value*="Downloads"]').value,
            maxConcurrent: document.querySelector('input[value="5"]').value,
            port: document.querySelector('input[value="6881"]').value,
            maxPeers: document.querySelector('input[value="50"]').value,
            upnp: document.querySelector('#upnp').checked,
            maxDownload: document.querySelector('input[placeholder="Sınırsız"]').value,
            maxUpload: document.querySelectorAll('input[placeholder="Sınırsız"]')[1].value
        };
        
        // Save to localStorage (in a real app, this would be sent to backend)
        localStorage.setItem('bitswap-settings', JSON.stringify(settings));
        
        this.showNotification('Ayarlar kaydedildi', 'success');
    }
    
    showNotification(message, type = 'info') {
        // Create notification element
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
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
            animation: slideInRight 0.3s ease;
            box-shadow: var(--shadow-lg);
        `;
        
        // Add icon based on type
        const iconClass = {
            success: 'fa-check-circle text-success',
            warning: 'fa-exclamation-triangle text-warning',
            error: 'fa-times-circle text-danger',
            info: 'fa-info-circle text-info'
        };
        
        notification.innerHTML = `
            <i class="fas ${iconClass[type] || iconClass.info}"></i>
            <span style="margin-left: 0.5rem;">${message}</span>
        `;
        
        document.body.appendChild(notification);
        
        // Remove after 3 seconds
        setTimeout(() => {
            notification.style.animation = 'slideOutRight 0.3s ease';
            setTimeout(() => notification.remove(), 300);
        }, 3000);
    }
    
    startStatsUpdater() {
        // Update stats every 2 seconds with random values (simulated)
        setInterval(() => {
            this.stats.downloadSpeed += (Math.random() - 0.5) * 200000; // ±100KB/s
            this.stats.uploadSpeed += (Math.random() - 0.5) * 100000;   // ±50KB/s
            
            // Keep values positive and realistic
            this.stats.downloadSpeed = Math.max(0, Math.min(this.stats.downloadSpeed, 10000000));
            this.stats.uploadSpeed = Math.max(0, Math.min(this.stats.uploadSpeed, 5000000));
            
            // Randomly update peers and progress
            this.peers.forEach(peer => {
                peer.downloadSpeed += (Math.random() - 0.5) * 50000;
                peer.uploadSpeed += (Math.random() - 0.5) * 50000;
                peer.downloadSpeed = Math.max(0, peer.downloadSpeed);
                peer.uploadSpeed = Math.max(0, peer.uploadSpeed);
            });
            
            // Update torrent progress
            this.torrents.forEach(torrent => {
                if (torrent.status === 'downloading' && torrent.progress < 100) {
                    torrent.progress += Math.random() * 0.5;
                    if (torrent.progress >= 100) {
                        torrent.progress = 100;
                        torrent.status = 'completed';
                    }
                }
            });
            
            if (this.activeTab === 'dashboard') {
                this.updateStatsCards();
            } else if (this.activeTab === 'peers') {
                this.renderPeers();
            } else if (this.activeTab === 'torrents') {
                this.renderTorrents();
            }
        }, 2000);
    }
    
    // Utility functions
    formatBytes(bytes) {
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        if (bytes === 0) return '0 B';
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return Math.round(bytes / Math.pow(1024, i) * 10) / 10 + ' ' + sizes[i];
    }
    
    formatSpeed(bytesPerSecond) {
        return this.formatBytes(bytesPerSecond) + '/s';
    }
    
    getFileIcon(filename) {
        const ext = filename.split('.').pop().toLowerCase();
        const iconMap = {
            'txt': 'fas fa-file-text',
            'pdf': 'fas fa-file-pdf',
            'doc': 'fas fa-file-word',
            'docx': 'fas fa-file-word',
            'xls': 'fas fa-file-excel',
            'xlsx': 'fas fa-file-excel',
            'ppt': 'fas fa-file-powerpoint',
            'pptx': 'fas fa-file-powerpoint',
            'zip': 'fas fa-file-archive',
            'rar': 'fas fa-file-archive',
            '7z': 'fas fa-file-archive',
            'iso': 'fas fa-file-archive',
            'mp3': 'fas fa-music',
            'wav': 'fas fa-music',
            'flac': 'fas fa-music',
            'mp4': 'fas fa-film',
            'avi': 'fas fa-film',
            'mkv': 'fas fa-film',
            'jpg': 'fas fa-image',
            'png': 'fas fa-image',
            'gif': 'fas fa-image',
        };
        return iconMap[ext] || 'fas fa-file';
    }
    
    getStatusText(status) {
        const statusMap = {
            'downloading': 'İndiriliyor',
            'seeding': 'Seeding',
            'completed': 'Tamamlandı',
            'paused': 'Duraklatıldı'
        };
        return statusMap[status] || status;
    }
    
    // Mock data generators
    getMockTorrents() {
        return [
            {
                name: 'ornek-dosya.txt',
                size: 112,
                status: 'seeding',
                progress: 100,
                peers: 3
            },
            {
                name: 'ubuntu-22.04.iso',
                size: 4700000000,
                status: 'downloading',
                progress: 67,
                peers: 12
            },
            {
                name: 'Music Collection 2024',
                size: 1200000000,
                status: 'completed',
                progress: 100,
                peers: 8
            }
        ];
    }
    
    getMockPeers() {
        return [
            {
                ip: '192.168.1.105',
                port: 6881,
                flag: '🇹🇷',
                client: 'BitSwapTorrent 0.1.0',
                downloadSpeed: 1200000,
                uploadSpeed: 340000
            },
            {
                ip: '88.247.123.45',
                port: 6881,
                flag: '🇺🇸',
                client: 'qBittorrent 4.5.2',
                downloadSpeed: 890000,
                uploadSpeed: 156000
            },
            {
                ip: '173.82.19.201',
                port: 51413,
                flag: '🇬🇧',
                client: 'Transmission 3.0',
                downloadSpeed: 2100000,
                uploadSpeed: 780000
            }
        ];
    }
    
    getMockStats() {
        return {
            downloadSpeed: 2400000,
            uploadSpeed: 847000,
            connectedPeers: 23,
            activeTorrents: 3
        };
    }
    
    closeAllModals() {
        document.querySelectorAll('.modal').forEach(modal => {
            modal.classList.remove('show');
        });
    }
}

// Modal functions (global)
function showAddTorrentModal() {
    document.getElementById('addTorrentModal').classList.add('show');
}

function closeModal(modalId) {
    document.getElementById(modalId).classList.remove('show');
}

function handleDragOver(e) {
    e.preventDefault();
    e.currentTarget.classList.add('drag-over');
}

function handleDrop(e) {
    e.preventDefault();
    e.currentTarget.classList.remove('drag-over');
    
    const files = Array.from(e.dataTransfer.files);
    handleFiles(files);
}

function handleFileSelect(e) {
    const files = Array.from(e.target.files);
    handleFiles(files);
}

function handleFiles(files) {
    files.forEach(file => {
        if (file.name.endsWith('.bwt') || file.name.endsWith('.torrent')) {
            // In a real app, this would upload the file
            app.showNotification(`${file.name} yüklendi`, 'success');
            closeModal('addTorrentModal');
            
            // Add mock torrent to list
            app.torrents.push({
                name: file.name.replace(/\.(bwt|torrent)$/, ''),
                size: Math.random() * 1000000000,
                status: 'downloading',
                progress: 0,
                peers: Math.floor(Math.random() * 20)
            });
            
            if (app.activeTab === 'torrents') {
                app.renderTorrents();
            }
        } else {
            app.showNotification('Yalnızca .bwt ve .torrent dosyaları desteklenir', 'error');
        }
    });
}

function addTorrent() {
    const magnetInput = document.getElementById('magnetInput');
    const magnetUrl = magnetInput.value.trim();
    
    if (magnetUrl && magnetUrl.startsWith('magnet:')) {
        // Parse magnet URL and add torrent
        const name = extractNameFromMagnet(magnetUrl) || 'Unknown Torrent';
        
        app.torrents.push({
            name: name,
            size: Math.random() * 1000000000,
            status: 'downloading',
            progress: 0,
            peers: Math.floor(Math.random() * 20)
        });
        
        app.showNotification(`${name} eklendi`, 'success');
        closeModal('addTorrentModal');
        magnetInput.value = '';
        
        if (app.activeTab === 'torrents') {
            app.renderTorrents();
        }
    } else {
        app.showNotification('Geçerli bir magnet link girin', 'error');
    }
}

function extractNameFromMagnet(magnetUrl) {
    const match = magnetUrl.match(/dn=([^&]+)/);
    return match ? decodeURIComponent(match[1]) : null;
}

// Add CSS for animations
const animationStyles = `
@keyframes slideInRight {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
}

@keyframes slideOutRight {
    from { transform: translateX(0); opacity: 1; }
    to { transform: translateX(100%); opacity: 0; }
}
`;

const styleSheet = document.createElement('style');
styleSheet.textContent = animationStyles;
document.head.appendChild(styleSheet);

// Initialize the app when DOM is loaded
let app;
document.addEventListener('DOMContentLoaded', () => {
    app = new BitSwapUI();
});
