<?php
header('Access-Control-Allow-Origin: *');
header('Access-Control-Allow-Methods: GET, OPTIONS');
header('Access-Control-Allow-Headers: Content-Type');

if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
    exit(0);
}

require_once '../config/database.php';

function downloadFile($hash) {
    try {
        $database = new Database();
        $pdo = $database->getConnection();
        
        // Get file info from database
        $stmt = $pdo->prepare("SELECT * FROM files WHERE file_hash = ? AND is_active = 1");
        $stmt->execute([$hash]);
        $file = $stmt->fetch();
        
        if (!$file) {
            throw new Exception('File not found');
        }
        
        // Check if file exists on disk
        if (!file_exists($file['file_path'])) {
            throw new Exception('File not found on disk');
        }
        
        // Update download count
        $stmt = $pdo->prepare("UPDATE files SET download_count = download_count + 1 WHERE file_hash = ?");
        $stmt->execute([$hash]);
        
        // Update peer info (register downloader)
        $downloaderIP = $_SERVER['HTTP_X_FORWARDED_FOR'] ?? $_SERVER['REMOTE_ADDR'] ?? 'unknown';
        $stmt = $pdo->prepare("
            INSERT OR REPLACE INTO peers (file_hash, peer_ip, peer_port, last_seen, download_speed) 
            VALUES (?, ?, ?, datetime('now'), ?)
        ");
        $stmt->execute([$hash, $downloaderIP, 8080, $file['file_size']]);
        
        // Set headers for download
        header('Content-Type: ' . ($file['mime_type'] ?: 'application/octet-stream'));
        header('Content-Disposition: attachment; filename="' . $file['original_name'] . '"');
        header('Content-Length: ' . $file['file_size']);
        header('Cache-Control: no-cache, must-revalidate');
        header('Expires: Sat, 26 Jul 1997 05:00:00 GMT');
        
        // Output file content
        readfile($file['file_path']);
        
        return true;
        
    } catch (Exception $e) {
        http_response_code(404);
        echo json_encode(['error' => $e->getMessage()]);
        return false;
    }
}

function getFileInfo($hash) {
    try {
        $database = new Database();
        $pdo = $database->getConnection();
        
        // Get file info
        $stmt = $pdo->prepare("SELECT * FROM files WHERE file_hash = ? AND is_active = 1");
        $stmt->execute([$hash]);
        $file = $stmt->fetch();
        
        if (!$file) {
            throw new Exception('File not found');
        }
        
        // Get peer count
        $stmt = $pdo->prepare("SELECT COUNT(*) as peer_count FROM peers WHERE file_hash = ?");
        $stmt->execute([$hash]);
        $peerInfo = $stmt->fetch();
        
        return [
            'success' => true,
            'file' => [
                'hash' => $file['file_hash'],
                'name' => $file['original_name'],
                'size' => intval($file['file_size']),
                'mime_type' => $file['mime_type'],
                'upload_time' => $file['upload_time'],
                'download_count' => intval($file['download_count']),
                'description' => $file['description'],
                'tags' => $file['tags'] ? explode(',', $file['tags']) : [],
                'peer_count' => intval($peerInfo['peer_count']),
                'download_url' => "download.php?hash=" . $hash,
                'magnet_url' => "magnet:?xt=urn:sha256:" . $hash . "&dn=" . urlencode($file['original_name']) . "&xl=" . $file['file_size']
            ]
        ];
        
    } catch (Exception $e) {
        return [
            'success' => false,
            'error' => $e->getMessage()
        ];
    }
}

// Handle request
if (isset($_GET['hash'])) {
    $hash = $_GET['hash'];
    
    // If info parameter is set, return file info as JSON
    if (isset($_GET['info'])) {
        header('Content-Type: application/json');
        echo json_encode(getFileInfo($hash));
    } else {
        // Download file
        downloadFile($hash);
    }
} else {
    http_response_code(400);
    echo json_encode(['error' => 'File hash not provided']);
}
?>
