<?php
// Database configuration
class Database {
    private $host = 'localhost';
    private $db_name = 'bitswap_torrent';
    private $username = 'root';
    private $password = '';
    private $charset = 'utf8mb4';
    private $pdo;
    
    public function getConnection() {
        if ($this->pdo === null) {
            try {
                $dsn = "mysql:host={$this->host};dbname={$this->db_name};charset={$this->charset}";
                $options = [
                    PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
                    PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
                    PDO::ATTR_EMULATE_PREPARES => false,
                ];
                
                $this->pdo = new PDO($dsn, $this->username, $this->password, $options);
                $this->createTables();
                
            } catch (PDOException $e) {
                // SQLite fallback for easier setup
                try {
                    $this->pdo = new PDO('sqlite:' . __DIR__ . '/../database.sqlite');
                    $this->pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
                    $this->createSQLiteTables();
                } catch (PDOException $e2) {
                    throw new Exception("Connection failed: " . $e2->getMessage());
                }
            }
        }
        
        return $this->pdo;
    }
    
    private function createTables() {
        $sql = "CREATE TABLE IF NOT EXISTS files (
            id INT AUTO_INCREMENT PRIMARY KEY,
            file_hash VARCHAR(64) UNIQUE NOT NULL,
            original_name VARCHAR(255) NOT NULL,
            file_path VARCHAR(512) NOT NULL,
            file_size BIGINT NOT NULL,
            mime_type VARCHAR(100),
            upload_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            download_count INT DEFAULT 0,
            is_active BOOLEAN DEFAULT TRUE,
            uploader_ip VARCHAR(45),
            description TEXT,
            tags VARCHAR(500)
        )";
        
        $this->pdo->exec($sql);
        
        $sql2 = "CREATE TABLE IF NOT EXISTS peers (
            id INT AUTO_INCREMENT PRIMARY KEY,
            file_hash VARCHAR(64),
            peer_ip VARCHAR(45),
            peer_port INT,
            last_seen TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            upload_speed BIGINT DEFAULT 0,
            download_speed BIGINT DEFAULT 0,
            INDEX(file_hash),
            FOREIGN KEY (file_hash) REFERENCES files(file_hash) ON DELETE CASCADE
        )";
        
        $this->pdo->exec($sql2);
    }
    
    private function createSQLiteTables() {
        $sql = "CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_hash TEXT UNIQUE NOT NULL,
            original_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            mime_type TEXT,
            upload_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            download_count INTEGER DEFAULT 0,
            is_active INTEGER DEFAULT 1,
            uploader_ip TEXT,
            description TEXT,
            tags TEXT
        )";
        
        $this->pdo->exec($sql);
        
        $sql2 = "CREATE TABLE IF NOT EXISTS peers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_hash TEXT,
            peer_ip TEXT,
            peer_port INTEGER,
            last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
            upload_speed INTEGER DEFAULT 0,
            download_speed INTEGER DEFAULT 0
        )";
        
        $this->pdo->exec($sql2);
    }
}
?>
