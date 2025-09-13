<?php
header('Content-Type: application/json');
header('Access-Control-Allow-Origin: *');
header('Access-Control-Allow-Methods: GET, OPTIONS');
header('Access-Control-Allow-Headers: Content-Type');

if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
    exit(0);
}

require_once '../config/database.php';

function getFiles($search = '', $limit = 50, $offset = 0) {
    try {
        $database = new Database();
        $pdo = $database->getConnection();
        
        // Build query
        $whereClause = "WHERE is_active = 1";
        $params = [];
        
        if (!empty($search)) {
            $whereClause .= " AND (original_name LIKE ? OR description LIKE ? OR tags LIKE ?)";
            $searchParam = "%{$search}%";
            $params = [$searchParam, $searchParam, $searchParam];
        }
        
        // Get total count
        $countSql = "SELECT COUNT(*) as total FROM files " . $whereClause;
        $stmt = $pdo->prepare($countSql);
        $stmt->execute($params);
        $total = $stmt->fetch()['total'];
        
        // Get files with peer count
        $sql = "SELECT f.*, COUNT(p.id) as peer_count 
                FROM files f 
                LEFT JOIN peers p ON f.file_hash = p.file_hash 
                " . $whereClause . " 
                GROUP BY f.id 
                ORDER BY f.upload_time DESC 
                LIMIT ? OFFSET ?";
        
        $params[] = $limit;
        $params[] = $offset;
        
        $stmt = $pdo->prepare($sql);
        $stmt->execute($params);
        $files = $stmt->fetchAll();
        
        // Format files for response
        $formattedFiles = [];
        foreach ($files as $file) {
            $formattedFiles[] = [
                'hash' => $file['file_hash'],
                'name' => $file['original_name'],
                'size' => intval($file['file_size']),
                'mime_type' => $file['mime_type'],
                'upload_time' => $file['upload_time'],
                'download_count' => intval($file['download_count']),
                'description' => $file['description'],
                'tags' => $file['tags'] ? explode(',', $file['tags']) : [],
                'peer_count' => intval($file['peer_count']),
                'download_url' => "download.php?hash=" . $file['file_hash'],
                'magnet_url' => "magnet:?xt=urn:sha256:" . $file['file_hash'] . "&dn=" . urlencode($file['original_name']) . "&xl=" . $file['file_size'],
                'status' => $file['peer_count'] > 0 ? 'seeding' : 'completed'
            ];
        }
        
        return [
            'success' => true,
            'files' => $formattedFiles,
            'total' => intval($total),
            'limit' => $limit,
            'offset' => $offset
        ];
        
    } catch (Exception $e) {
        return [
            'success' => false,
            'error' => $e->getMessage()
        ];
    }
}

function getStats() {
    try {
        $database = new Database();
        $pdo = $database->getConnection();
        
        // Get various stats
        $stmt = $pdo->prepare("SELECT COUNT(*) as total_files FROM files WHERE is_active = 1");
        $stmt->execute();
        $totalFiles = $stmt->fetch()['total_files'];
        
        $stmt = $pdo->prepare("SELECT SUM(file_size) as total_size FROM files WHERE is_active = 1");
        $stmt->execute();
        $totalSize = $stmt->fetch()['total_size'] ?: 0;
        
        $stmt = $pdo->prepare("SELECT COUNT(DISTINCT peer_ip) as active_peers FROM peers WHERE last_seen > datetime('now', '-1 hour')");
        $stmt->execute();
        $activePeers = $stmt->fetch()['active_peers'];
        
        $stmt = $pdo->prepare("SELECT SUM(download_count) as total_downloads FROM files WHERE is_active = 1");
        $stmt->execute();
        $totalDownloads = $stmt->fetch()['total_downloads'] ?: 0;
        
        return [
            'success' => true,
            'stats' => [
                'total_files' => intval($totalFiles),
                'total_size' => intval($totalSize),
                'active_peers' => intval($activePeers),
                'total_downloads' => intval($totalDownloads),
                'avg_download_speed' => 2400000, // Mock value
                'avg_upload_speed' => 847000     // Mock value
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
$action = $_GET['action'] ?? 'list';

switch ($action) {
    case 'list':
        $search = $_GET['search'] ?? '';
        $limit = min(intval($_GET['limit'] ?? 50), 100);
        $offset = intval($_GET['offset'] ?? 0);
        echo json_encode(getFiles($search, $limit, $offset));
        break;
        
    case 'stats':
        echo json_encode(getStats());
        break;
        
    default:
        echo json_encode(['success' => false, 'error' => 'Invalid action']);
}
?>
