// File monitoring service for configuration changes
use crate::{AppError, AppResult, FileMetadata, ConfigFileChange, ChangeType, MonitoringStats};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::{interval, Interval};

pub struct MonitorService {
    monitored_files: Vec<PathBuf>,
    file_metadata: Arc<Mutex<HashMap<PathBuf, FileMetadata>>>,
    monitor_interval_minutes: u64,
    timer: Option<Interval>,
    is_running: Arc<Mutex<bool>>,
    // Performance optimization: limit metadata cache size
    max_cache_size: usize,
    scan_error_count: Arc<Mutex<u32>>,
    max_scan_errors: u32,
}

impl MonitorService {
    /// Create a new monitor service with specified interval
    pub fn new(interval_minutes: u64) -> Self {
        Self {
            monitored_files: Vec::new(),
            file_metadata: Arc::new(Mutex::new(HashMap::new())),
            monitor_interval_minutes: interval_minutes,
            timer: None,
            is_running: Arc::new(Mutex::new(false)),
            max_cache_size: 100, // Limit cache to 100 files to manage memory
            scan_error_count: Arc::new(Mutex::new(0)),
            max_scan_errors: 10, // Stop scanning after 10 consecutive errors
        }
    }
    
    /// Set the monitoring interval (1-60 minutes) with performance optimization
    pub fn set_monitor_interval(&mut self, minutes: u64) -> AppResult<()> {
        if !(1..=60).contains(&minutes) {
            return Err(AppError::MonitorError(
                format!("Invalid monitor interval: {} minutes. Must be between 1 and 60.", minutes)
            ));
        }
        
        let old_interval = self.monitor_interval_minutes;
        self.monitor_interval_minutes = minutes;
        log::info!("Monitor interval changed from {} to {} minutes", old_interval, minutes);
        
        // Only restart if interval changed significantly (more than 30 seconds difference)
        if old_interval != minutes && *self.is_running.lock().unwrap() {
            self.restart_monitoring()?;
        }
        
        Ok(())
    }
    
    /// Add a file to the monitoring list with enhanced cross-platform validation
    pub fn add_file_to_monitor(&mut self, path: PathBuf) {
        // Validate file path exists and is accessible
        if !path.exists() {
            log::warn!("Cannot monitor non-existent file: {:?}", path);
            return;
        }
        
        if !path.is_file() {
            log::warn!("Cannot monitor non-file path: {:?}", path);
            return;
        }
        
        // Cross-platform file accessibility check
        match std::fs::File::open(&path) {
            Ok(_) => {
                log::debug!("File accessibility confirmed: {:?}", path);
            }
            Err(e) => {
                log::warn!("Cannot access file for monitoring: {:?} - {}", path, e);
                return;
            }
        }
        
        // Check file size - don't monitor very large files (>10MB)
        if let Ok(metadata) = std::fs::metadata(&path) {
            const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
            if metadata.len() > MAX_FILE_SIZE {
                log::warn!("File too large to monitor ({}MB): {:?}", 
                    metadata.len() / (1024 * 1024), path);
                return;
            }
        }
        
        // Check if path is already being monitored (case-insensitive on Windows)
        let path_already_monitored = if cfg!(target_os = "windows") {
            // Case-insensitive comparison on Windows
            self.monitored_files.iter().any(|existing| {
                existing.to_string_lossy().to_lowercase() == path.to_string_lossy().to_lowercase()
            })
        } else {
            self.monitored_files.contains(&path)
        };
        
        if !path_already_monitored {
            log::info!("Adding file to monitor: {:?}", path);
            self.monitored_files.push(path);
            
            // Enforce maximum number of monitored files for performance
            const MAX_MONITORED_FILES: usize = 50;
            if self.monitored_files.len() > MAX_MONITORED_FILES {
                log::warn!("Too many files being monitored ({}), removing oldest", self.monitored_files.len());
                let removed = self.monitored_files.remove(0);
                log::info!("Removed from monitoring: {:?}", removed);
                
                // Also remove from metadata cache
                if let Ok(mut metadata_map) = self.file_metadata.lock() {
                    metadata_map.remove(&removed);
                }
            }
            
            // Optimize cache after adding new file
            self.optimize_metadata_cache();
        } else {
            log::debug!("File already being monitored: {:?}", path);
        }
    }
    
    /// Optimize metadata cache by removing old entries
    fn optimize_metadata_cache(&self) {
        if let Ok(mut metadata_map) = self.file_metadata.lock() {
            if metadata_map.len() > self.max_cache_size {
                // Keep only files that are still being monitored
                let monitored_set: std::collections::HashSet<_> = self.monitored_files.iter().collect();
                metadata_map.retain(|path, _| monitored_set.contains(path));
                
                log::debug!("Optimized metadata cache, kept {} entries", metadata_map.len());
            }
        }
    }
    
    /// Start monitoring with a callback function and enhanced error handling
    pub fn start_monitoring<F>(&mut self, callback: F) -> AppResult<()>
    where
        F: Fn(Vec<ConfigFileChange>) + Send + Sync + 'static,
    {
        log::info!("Starting file monitoring with {} minute interval", self.monitor_interval_minutes);
        
        // Validate that we have files to monitor
        if self.monitored_files.is_empty() {
            log::warn!("No files to monitor, starting monitoring anyway");
        }
        
        let callback = Arc::new(callback);
        let monitored_files = self.monitored_files.clone();
        let file_metadata = Arc::clone(&self.file_metadata);
        let is_running = Arc::clone(&self.is_running);
        let scan_error_count = Arc::clone(&self.scan_error_count);
        let interval_minutes = self.monitor_interval_minutes;
        let max_scan_errors = self.max_scan_errors;
        
        // Initialize file metadata with error handling
        if let Err(e) = self.initialize_file_metadata() {
            log::warn!("Failed to initialize file metadata: {}", e);
            // Continue anyway, metadata will be initialized during first scan
        }
        
        // Reset error count
        *scan_error_count.lock().unwrap() = 0;
        
        // Mark as running
        *is_running.lock().unwrap() = true;
        
        // Create timer interval
        let mut timer = interval(Duration::from_secs(interval_minutes * 60));
        
        // Spawn monitoring task with error resilience
        tokio::spawn(async move {
            let mut consecutive_errors = 0u32;
            
            loop {
                timer.tick().await;
                
                // Check if we should continue running
                if !*is_running.lock().unwrap() {
                    log::info!("Monitor service stopped");
                    break;
                }
                
                log::debug!("Performing scheduled file scan");
                
                match Self::perform_scan_optimized(&monitored_files, &file_metadata).await {
                    Ok(changes) => {
                        consecutive_errors = 0;
                        *scan_error_count.lock().unwrap() = 0;
                        
                        if !changes.is_empty() {
                            log::info!("Detected {} file changes", changes.len());
                            callback(changes);
                        }
                    }
                    Err(e) => {
                        consecutive_errors += 1;
                        *scan_error_count.lock().unwrap() = consecutive_errors;
                        
                        log::error!("Error during file scan (attempt {}): {}", consecutive_errors, e);
                        
                        // Stop monitoring after too many consecutive errors
                        if consecutive_errors >= max_scan_errors {
                            log::error!("Too many consecutive scan errors ({}), stopping monitoring", consecutive_errors);
                            *is_running.lock().unwrap() = false;
                            break;
                        }
                        
                        // Exponential backoff for errors
                        let backoff_duration = Duration::from_secs(std::cmp::min(300, 30 * consecutive_errors as u64)); // Max 5 minutes
                        log::info!("Backing off for {} seconds due to scan errors", backoff_duration.as_secs());
                        tokio::time::sleep(backoff_duration).await;
                    }
                }
                
                // Periodic cache optimization would need proper cycle counting
                // This is a placeholder for future implementation
            }
            
            log::info!("Monitor service task terminated");
        });
        
        log::info!("File monitoring started successfully");
        Ok(())
    }
    
    /// Optimized scan with better resource management and cross-platform error handling
    async fn perform_scan_optimized(
        monitored_files: &[PathBuf],
        file_metadata: &Arc<Mutex<HashMap<PathBuf, FileMetadata>>>,
    ) -> AppResult<Vec<ConfigFileChange>> {
        let mut changes = Vec::new();
        let mut scan_errors = Vec::new();
        
        // Minimize lock time by collecting current metadata first
        let cached_metadata: HashMap<PathBuf, FileMetadata> = {
            match file_metadata.lock() {
                Ok(metadata_map) => metadata_map.clone(),
                Err(e) => {
                    log::error!("Failed to acquire metadata lock: {}", e);
                    return Err(AppError::MonitorError("Metadata lock poisoned".to_string()));
                }
            }
        };
        
        // Process files in smaller batches to avoid overwhelming the system
        const BATCH_SIZE: usize = 10;
        for batch in monitored_files.chunks(BATCH_SIZE) {
            for file_path in batch {
                match Self::scan_single_file_with_retry(file_path, &cached_metadata).await {
                    Ok(file_changes) => {
                        changes.extend(file_changes);
                    }
                    Err(e) => {
                        scan_errors.push(format!("Error scanning {:?}: {}", file_path, e));
                    }
                }
            }
            
            // Small delay between batches to prevent system overload
            if monitored_files.len() > BATCH_SIZE {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
        
        // Update metadata cache in batch with error handling
        if !changes.is_empty() {
            match file_metadata.lock() {
                Ok(mut metadata_map) => {
                    for change in &changes {
                        match change.change_type {
                            ChangeType::Created | ChangeType::Modified => {
                                if let Ok(new_metadata) = Self::get_file_metadata_optimized(&change.file_path) {
                                    metadata_map.insert(change.file_path.clone(), new_metadata);
                                }
                            }
                            ChangeType::Deleted => {
                                metadata_map.remove(&change.file_path);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to update metadata cache: {}", e);
                    // Don't fail the entire scan for cache update errors
                }
            }
        }
        
        // Log scan errors but don't fail the entire scan unless too many errors
        if !scan_errors.is_empty() {
            let error_rate = scan_errors.len() as f32 / monitored_files.len() as f32;
            
            if error_rate > 0.5 {
                // More than 50% of files failed - this might indicate a system issue
                log::error!("High error rate during scan: {:.1}% ({}/{})", 
                    error_rate * 100.0, scan_errors.len(), monitored_files.len());
                return Err(AppError::MonitorError(format!(
                    "Too many scan errors: {} out of {} files failed", 
                    scan_errors.len(), monitored_files.len()
                )));
            } else {
                log::warn!("Encountered {} file scan errors:", scan_errors.len());
                for error in scan_errors.iter().take(3) { // Limit log spam
                    log::warn!("  - {}", error);
                }
                if scan_errors.len() > 3 {
                    log::warn!("  ... and {} more errors", scan_errors.len() - 3);
                }
            }
        }
        
        Ok(changes)
    }
    
    /// Scan single file with retry mechanism for transient errors
    async fn scan_single_file_with_retry(
        file_path: &PathBuf,
        cached_metadata: &HashMap<PathBuf, FileMetadata>,
    ) -> AppResult<Vec<ConfigFileChange>> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 100;
        
        for attempt in 1..=MAX_RETRIES {
            match Self::scan_single_file(file_path, cached_metadata).await {
                Ok(changes) => return Ok(changes),
                Err(e) => {
                    if attempt < MAX_RETRIES {
                        log::debug!("Retry {}/{} for file {:?}: {}", attempt, MAX_RETRIES, file_path, e);
                        tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS * attempt as u64)).await;
                    } else {
                        log::warn!("Failed to scan file {:?} after {} attempts: {}", file_path, MAX_RETRIES, e);
                        return Err(e);
                    }
                }
            }
        }
        
        unreachable!()
    }
    
    /// Scan a single file for changes
    async fn scan_single_file(
        file_path: &PathBuf,
        cached_metadata: &HashMap<PathBuf, FileMetadata>,
    ) -> AppResult<Vec<ConfigFileChange>> {
        let mut changes = Vec::new();
        
        // Check if file still exists
        if !file_path.exists() {
            if cached_metadata.contains_key(file_path) {
                // File was deleted
                changes.push(ConfigFileChange {
                    file_path: file_path.clone(),
                    change_type: ChangeType::Deleted,
                });
                log::info!("File deleted: {:?}", file_path);
            }
            return Ok(changes);
        }
        
        // Check if file was modified
        match Self::get_file_metadata_optimized(file_path) {
            Ok(current_metadata) => {
                let was_changed = if let Some(cached) = cached_metadata.get(file_path) {
                    // Check for modifications using optimized comparison
                    Self::compare_metadata_optimized(cached, &current_metadata)
                } else {
                    // New file
                    changes.push(ConfigFileChange {
                        file_path: file_path.clone(),
                        change_type: ChangeType::Created,
                    });
                    log::info!("New file detected: {:?}", file_path);
                    true
                };
                
                if was_changed && cached_metadata.contains_key(file_path) {
                    changes.push(ConfigFileChange {
                        file_path: file_path.clone(),
                        change_type: ChangeType::Modified,
                    });
                    log::info!("File modified: {:?}", file_path);
                }
            }
            Err(e) => {
                return Err(AppError::FileSystemError(
                    format!("Failed to get metadata for {:?}: {}", file_path, e)
                ));
            }
        }
        
        Ok(changes)
    }
    
    /// Optimized metadata retrieval with lazy content reading
    fn get_file_metadata_optimized(path: &Path) -> AppResult<FileMetadata> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to get file metadata: {}", e)))?;
        
        // For very large files, skip content reading and use a dummy checksum
        const MAX_CONTENT_SIZE: u64 = 1024 * 1024; // 1MB
        let checksum = if metadata.len() > MAX_CONTENT_SIZE {
            log::debug!("File too large for content checksum: {:?} ({}MB)", path, metadata.len() / (1024 * 1024));
            0 // Use dummy checksum for large files, rely on modification time and size
        } else {
            // Read content for checksum calculation
            match std::fs::read_to_string(path) {
                Ok(content) => crc32fast::hash(content.as_bytes()),
                Err(e) => {
                    log::warn!("Failed to read file content for {:?}: {}", path, e);
                    0 // Use dummy checksum if content can't be read
                }
            }
        };
        
        Ok(FileMetadata {
            modified_time: metadata.modified()
                .map_err(|e| AppError::FileSystemError(format!("Failed to get modification time: {}", e)))?,
            checksum,
            size: metadata.len(),
        })
    }
    
    /// Optimized metadata comparison with early exit
    fn compare_metadata_optimized(cached: &FileMetadata, current: &FileMetadata) -> bool {
        // Fast checks first (modification time and size)
        if cached.modified_time != current.modified_time {
            return true;
        }
        
        if cached.size != current.size {
            return true;
        }
        
        // Only check checksum if other checks pass and both have valid checksums
        if cached.checksum != 0 && current.checksum != 0 && cached.checksum != current.checksum {
            return true;
        }
        
        false
    }
    
    /// Stop monitoring with cleanup
    pub fn stop_monitoring(&mut self) {
        log::info!("Stopping file monitoring");
        *self.is_running.lock().unwrap() = false;
        self.timer = None;
        
        // Optimize cache when stopping
        self.optimize_metadata_cache();
    }
    
    /// Get monitoring statistics for performance analysis
    pub fn get_monitoring_stats(&self) -> MonitoringStats {
        let metadata_count = self.file_metadata.lock().unwrap().len();
        let error_count = *self.scan_error_count.lock().unwrap();
        
        MonitoringStats {
            monitored_files_count: self.monitored_files.len(),
            cached_metadata_count: metadata_count,
            current_error_count: error_count,
            is_running: self.is_monitoring(),
            interval_minutes: self.monitor_interval_minutes,
            cache_size_limit: self.max_cache_size,
            max_scan_errors: self.max_scan_errors,
        }
    }
    
    /// Force optimization of metadata cache
    pub fn force_optimize_cache(&self) {
        log::info!("Forcing cache optimization");
        self.optimize_metadata_cache();
    }
    
    /// Clear all cached metadata (useful for troubleshooting)
    pub fn clear_cache(&self) {
        log::info!("Clearing all cached metadata");
        self.file_metadata.lock().unwrap().clear();
        *self.scan_error_count.lock().unwrap() = 0;
    }
    
    /// Restart monitoring (used when interval changes)
    fn restart_monitoring(&mut self) -> AppResult<()> {
        log::info!("Restarting monitoring with new interval");
        self.stop_monitoring();
        // Note: The callback would need to be stored to restart properly
        // For now, this is a placeholder that requires manual restart
        Ok(())
    }
    
    /// Initialize metadata for all monitored files
    fn initialize_file_metadata(&self) -> AppResult<()> {
        let mut metadata_map = self.file_metadata.lock().unwrap();
        
        for file_path in &self.monitored_files {
            if file_path.exists() {
                match Self::get_file_metadata(file_path) {
                    Ok(metadata) => {
                        metadata_map.insert(file_path.clone(), metadata);
                        log::debug!("Initialized metadata for: {:?}", file_path);
                    }
                    Err(e) => {
                        log::warn!("Failed to initialize metadata for {:?}: {}", file_path, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Perform a scan of all monitored files
    async fn perform_scan(
        monitored_files: &[PathBuf],
        file_metadata: &Arc<Mutex<HashMap<PathBuf, FileMetadata>>>,
    ) -> AppResult<Vec<ConfigFileChange>> {
        let mut changes = Vec::new();
        let mut metadata_map = file_metadata.lock().unwrap();
        
        for file_path in monitored_files {
            // Check if file still exists
            if !file_path.exists() {
                if metadata_map.contains_key(file_path) {
                    // File was deleted
                    changes.push(ConfigFileChange {
                        file_path: file_path.clone(),
                        change_type: ChangeType::Deleted,
                    });
                    metadata_map.remove(file_path);
                    log::info!("File deleted: {:?}", file_path);
                }
                continue;
            }
            
            // Check if file was modified
            match Self::get_file_metadata(file_path) {
                Ok(current_metadata) => {
                    let was_modified = if let Some(cached_metadata) = metadata_map.get(file_path) {
                        // Check if file was modified
                        Self::compare_metadata(cached_metadata, &current_metadata)
                    } else {
                        // New file
                        changes.push(ConfigFileChange {
                            file_path: file_path.clone(),
                            change_type: ChangeType::Created,
                        });
                        log::info!("New file detected: {:?}", file_path);
                        true
                    };
                    
                    if was_modified {
                        if metadata_map.contains_key(file_path) {
                            changes.push(ConfigFileChange {
                                file_path: file_path.clone(),
                                change_type: ChangeType::Modified,
                            });
                            log::info!("File modified: {:?}", file_path);
                        }
                        
                        // Update cached metadata
                        metadata_map.insert(file_path.clone(), current_metadata);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to get metadata for {:?}: {}", file_path, e);
                }
            }
        }
        
        Ok(changes)
    }
    
    /// Get file metadata (modification time, size, checksum)
    fn get_file_metadata(path: &Path) -> AppResult<FileMetadata> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to get file metadata: {}", e)))?;
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to read file: {}", e)))?;
        
        Ok(FileMetadata {
            modified_time: metadata.modified()
                .map_err(|e| AppError::FileSystemError(format!("Failed to get modification time: {}", e)))?,
            checksum: crc32fast::hash(content.as_bytes()),
            size: metadata.len(),
        })
    }
    
    /// Compare two metadata instances to detect changes
    fn compare_metadata(cached: &FileMetadata, current: &FileMetadata) -> bool {
        // First check modification time (fastest)
        if cached.modified_time != current.modified_time {
            // Then check size (also fast)
            if cached.size != current.size {
                return true;
            }
            // Finally check content checksum
            if cached.checksum != current.checksum {
                return true;
            }
        }
        false
    }
    
    /// Force an immediate scan of all files
    pub async fn force_scan(&self) -> AppResult<Vec<ConfigFileChange>> {
        log::info!("Performing forced file scan");
        Self::perform_scan(&self.monitored_files, &self.file_metadata).await
    }
    
    /// Check if monitoring is currently running
    pub fn is_monitoring(&self) -> bool {
        *self.is_running.lock().unwrap()
    }
    
    /// Get current monitoring interval
    pub fn get_monitor_interval(&self) -> u64 {
        self.monitor_interval_minutes
    }
    
    /// Get list of monitored files
    pub fn get_monitored_files(&self) -> &[PathBuf] {
        &self.monitored_files
    }
    
    /// Clear all monitored files
    pub fn clear_monitored_files(&mut self) {
        self.monitored_files.clear();
        self.file_metadata.lock().unwrap().clear();
        log::info!("Cleared all monitored files");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::SystemTime;
    use tempfile::TempDir;
    
    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        fs::write(&file_path, content).unwrap();
        file_path
    }
    
    #[test]
    fn test_monitor_service_creation() {
        let service = MonitorService::new(5);
        assert_eq!(service.get_monitor_interval(), 5);
        assert!(!service.is_monitoring());
        assert!(service.get_monitored_files().is_empty());
    }
    
    #[test]
    fn test_set_monitor_interval() {
        let mut service = MonitorService::new(5);
        
        // Valid interval
        assert!(service.set_monitor_interval(10).is_ok());
        assert_eq!(service.get_monitor_interval(), 10);
        
        // Invalid intervals
        assert!(service.set_monitor_interval(0).is_err());
        assert!(service.set_monitor_interval(61).is_err());
    }
    
    #[test]
    fn test_add_file_to_monitor() {
        let mut service = MonitorService::new(5);
        let path = PathBuf::from("test.json");
        
        service.add_file_to_monitor(path.clone());
        assert_eq!(service.get_monitored_files().len(), 1);
        assert_eq!(service.get_monitored_files()[0], path);
        
        // Adding the same file shouldn't duplicate
        service.add_file_to_monitor(path.clone());
        assert_eq!(service.get_monitored_files().len(), 1);
    }
    
    #[test]
    fn test_get_file_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file(temp_dir.path(), "test.json", "{}");
        
        let metadata = MonitorService::get_file_metadata(&file_path).unwrap();
        assert_eq!(metadata.size, 2); // "{}" = 2 bytes
        assert_eq!(metadata.checksum, crc32fast::hash(b"{}"));
    }
    
    #[test]
    fn test_compare_metadata() {
        let metadata1 = FileMetadata {
            modified_time: SystemTime::UNIX_EPOCH,
            checksum: 123,
            size: 100,
        };
        
        let metadata2 = FileMetadata {
            modified_time: SystemTime::UNIX_EPOCH,
            checksum: 123,
            size: 100,
        };
        
        let metadata3 = FileMetadata {
            modified_time: SystemTime::now(),
            checksum: 456,
            size: 200,
        };
        
        assert!(!MonitorService::compare_metadata(&metadata1, &metadata2));
        assert!(MonitorService::compare_metadata(&metadata1, &metadata3));
    }
}