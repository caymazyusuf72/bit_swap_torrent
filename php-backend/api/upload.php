<?php
header('Content-Type: application/json');
header('Access-Control-Allow-Origin: *');
header('Access-Control-Allow-Methods: POST, OPTIONS');
header('Access-Control-Allow-Headers: Content-Type');

if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
    exit(0);
}

require_once '../config/database.php';

function uploadFile() {
    try {
        $database = new Database();
        $pdo = $database->getConnection();
        
        // Check if file was uploaded
        if (!isset($_FILES['file']) || $_FILES['file']['error'] !== UPLOAD_ERR_OK) {
            throw new Exception('No file uploaded or upload error occurred');
        }
        
        $uploadedFile = $_FILES['file'];
        $originalName = $uploadedFile['name'];
        $tmpPath = $uploadedFile['tmp_name'];
        $fileSize = $uploadedFile['size'];
        $mimeType = mime_content_type($tmpPath);
        
        // Generate unique hash for file
        $fileContent = file_get_contents($tmpPath);
        $fileHash = hash('sha256', $fileContent);
        
        // Check if file already exists
        $stmt = $pdo->prepare("SELECT * FROM files WHERE file_hash = ?");
        $stmt->execute([$fileHash]);
        $existingFile = $stmt->fetch();
        
        if ($existingFile) {
            return [
                'success' => true,
                'message' => 'File already exists in network',
                'hash' => $fileHash,
                'download_url' => "download.php?hash=" . $fileHash,
                'magnet_url' => "magnet:?xt=urn:sha256:" . $fileHash . "&dn=" . urlencode($originalName),
                'existing' => true
            ];
        }
        
        // Create uploads directory if not exists
        $uploadsDir = '../uploads/';
        if (!is_dir($uploadsDir)) {
            mkdir($uploadsDir, 0755, true);
        }
        
        // Save file with hash as filename
        $fileName = $fileHash . '_' . preg_replace('/[^a-zA-Z0-9._-]/', '', $originalName);
        $filePath = $uploadsDir . $fileName;
        
        if (!move_uploaded_file($tmpPath, $filePath)) {
            throw new Exception('Failed to save uploaded file');
        }
        
        // Get uploader IP
        $uploaderIP = $_SERVER['HTTP_X_FORWARDED_FOR'] ?? $_SERVER['REMOTE_ADDR'] ?? 'unknown';
        
        // Get description and tags from POST data
        $description = $_POST['description'] ?? '';
        $tags = $_POST['tags'] ?? '';
        
        // Insert file info into database
        $stmt = $pdo->prepare("
            INSERT INTO files (file_hash, original_name, file_path, file_size, mime_type, uploader_ip, description, tags) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ");
        
        $stmt->execute([
            $fileHash,
            $originalName,
            $filePath,
            $fileSize,
            $mimeType,
            $uploaderIP,
            $description,
            $tags
        ]);
        
        // Register as initial peer
        $stmt = $pdo->prepare("
            INSERT INTO peers (file_hash, peer_ip, peer_port, upload_speed) 
            VALUES (?, ?, ?, ?)
        ");
        $stmt->execute([$fileHash, $uploaderIP, 8080, $fileSize]);
        
        return [
            'success' => true,
            'message' => 'File uploaded successfully',
            'hash' => $fileHash,
            'original_name' => $originalName,
            'size' => $fileSize,
            'download_url' => "download.php?hash=" . $fileHash,
            'magnet_url' => "magnet:?xt=urn:sha256:" . $fileHash . "&dn=" . urlencode($originalName) . "&xl=" . $fileSize,
            'share_url' => "http://" . $_SERVER['HTTP_HOST'] . dirname($_SERVER['REQUEST_URI']) . "/download.php?hash=" . $fileHash
        ];
        
    } catch (Exception $e) {
        return [
            'success' => false,
            'message' => $e->getMessage()
        ];
    }
}

// Handle request
if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    $result = uploadFile();
    echo json_encode($result);
} else {
    echo json_encode(['success' => false, 'message' => 'Invalid request method']);
}
?>
