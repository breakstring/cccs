// Configuration service for managing Claude Code profiles
use crate::{AppError, AppResult, Profile, ProfileStatus, FileMetadata, ProfileInfo, ValidationResult};
use crate::validation::JsonValidator;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration};

// Cache for profile metadata to improve performance
#[derive(Clone, Debug)]
struct ProfileCache {
    metadata: FileMetadata,
    content: String,
    last_updated: SystemTime,
}

pub struct ConfigService {
    claude_dir: PathBuf,
    profiles: Vec<Profile>,
    default_settings_path: PathBuf,
    // Performance optimization: cache file metadata and content
    profile_cache: HashMap<PathBuf, ProfileCache>,
    default_settings_cache: Option<(String, SystemTime)>,
    cache_ttl: Duration,
    // Validation framework
    validator: JsonValidator,
}

impl ConfigService {
    pub fn new(claude_dir: PathBuf) -> Self {
        let default_settings_path = claude_dir.join("settings.json");
        Self {
            claude_dir,
            profiles: Vec::new(),
            default_settings_path,
            profile_cache: HashMap::new(),
            default_settings_cache: None,
            cache_ttl: Duration::from_secs(60), // 1 minute cache TTL
            validator: JsonValidator::with_basic_rules(),
        }
    }
    
    
    /// Clear all caches when needed
    pub fn clear_cache(&mut self) {
        self.profile_cache.clear();
        self.default_settings_cache = None;
        log::debug!("Cleared all caches");
    }
    
    /// Check if cache entry is still valid
    fn is_cache_valid(&self, last_updated: SystemTime) -> bool {
        if let Ok(elapsed) = last_updated.elapsed() {
            elapsed < self.cache_ttl
        } else {
            false
        }
    }
    
    /// Get cached content or read from file with caching
    fn get_file_content_cached(&mut self, path: &Path) -> AppResult<String> {
        // Check if we have valid cached content
        if let Some(cache_entry) = self.profile_cache.get(path) {
            if self.is_cache_valid(cache_entry.last_updated) {
                // Check if file has been modified since cache
                if let Ok(metadata) = fs::metadata(path) {
                    if let Ok(modified) = metadata.modified() {
                        if modified <= cache_entry.metadata.modified_time {
                            log::debug!("Using cached content for: {:?}", path);
                            return Ok(cache_entry.content.clone());
                        }
                    }
                }
            }
        }
        
        // Cache miss or invalid - read from file and update cache
        log::debug!("Reading fresh content for: {:?}", path);
        let content = fs::read_to_string(path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to read file {:?}: {}", path, e)))?;
        
        // Update cache
        if let Ok(metadata) = self.get_file_metadata(path) {
            let cache_entry = ProfileCache {
                metadata,
                content: content.clone(),
                last_updated: SystemTime::now(),
            };
            self.profile_cache.insert(path.to_path_buf(), cache_entry);
        }
        
        Ok(content)
    }
    
    /// Get default settings with caching
    fn get_default_settings_cached(&mut self) -> AppResult<String> {
        // Check cache first
        if let Some((cached_content, cached_time)) = &self.default_settings_cache {
            if self.is_cache_valid(*cached_time) {
                // Check if file has been modified
                if let Ok(metadata) = fs::metadata(&self.default_settings_path) {
                    if let Ok(modified) = metadata.modified() {
                        if modified <= *cached_time {
                            log::debug!("Using cached default settings");
                            return Ok(cached_content.clone());
                        }
                    }
                }
            }
        }
        
        // Read fresh content
        log::debug!("Reading fresh default settings");
        let content = self.read_default_settings()?;
        self.default_settings_cache = Some((content.clone(), SystemTime::now()));
        
        Ok(content)
    }
    
    /// Scan for all profile configuration files in the Claude directory
    pub fn scan_profiles(&mut self) -> AppResult<Vec<Profile>> {
        let mut profiles = Vec::new();
        let mut scan_errors = Vec::new();
        
        log::info!("Scanning for profiles in: {:?}", self.claude_dir);
        
        // Validate Claude directory exists and is readable
        if !self.claude_dir.exists() {
            return Err(AppError::FileSystemError(
                format!("Claude directory does not exist: {:?}", self.claude_dir)
            ));
        }
        
        if !self.claude_dir.is_dir() {
            return Err(AppError::FileSystemError(
                format!("Claude path is not a directory: {:?}", self.claude_dir)
            ));
        }
        
        let entries = fs::read_dir(&self.claude_dir)
            .map_err(|e| AppError::FileSystemError(
                format!("Failed to read Claude directory {:?}: {}", self.claude_dir, e)
            ))?;
        
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    scan_errors.push(format!("Failed to read directory entry: {}", e));
                    continue;
                }
            };
            
            let path = entry.path();
            
            if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
                // Look for files with pattern "*.settings.json" but exclude "settings.json"
                if filename.ends_with(".settings.json") && filename != "settings.json" {
                    if let Some(profile_name) = filename.strip_suffix(".settings.json") {
                        // Validate profile name is not empty
                        if !profile_name.is_empty() {
                            match self.load_profile_optimized(profile_name, &path) {
                                Ok(profile) => {
                                    log::info!("Found profile: {} at {:?}", profile.name, path);
                                    profiles.push(profile);
                                }
                                Err(e) => {
                                    let error_msg = format!("Failed to load profile {}: {}", profile_name, e);
                                    log::warn!("{}", error_msg);
                                    scan_errors.push(error_msg);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Log scan errors but don't fail the entire operation
        if !scan_errors.is_empty() {
            log::warn!("Encountered {} errors during profile scan:", scan_errors.len());
            for error in &scan_errors {
                log::warn!("  - {}", error);
            }
        }
        
        // Update active status for all profiles
        match self.update_profile_status_optimized(&mut profiles) {
            Ok(()) => {
                log::info!("Successfully scanned {} profiles", profiles.len());
            }
            Err(e) => {
                log::error!("Failed to update profile status: {}", e);
                // Don't fail the scan, but log the error
            }
        }
        
        self.profiles = profiles.clone();
        Ok(profiles)
    }
    
    /// Load a single profile from a file with performance optimizations
    fn load_profile_optimized(&mut self, name: &str, path: &Path) -> AppResult<Profile> {
        // Validate profile name
        if name.is_empty() {
            return Err(AppError::ConfigError("Profile name cannot be empty".to_string()));
        }
        
        if name.len() > 255 {
            return Err(AppError::ConfigError("Profile name too long (max 255 characters)".to_string()));
        }
        
        // Use cached content if available
        let content = self.get_file_content_cached(path)?;
        
        // Validate JSON format with detailed error reporting
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => {
                // JSON is valid
                log::debug!("Profile {} loaded successfully", name);
            }
            Err(e) => {
                return Err(AppError::ConfigError(
                    format!("Invalid JSON in profile {}: {} (line {}, column {})", 
                        name, 
                        e.to_string(),
                        e.line(),
                        e.column()
                    )
                ));
            }
        }
        
        Ok(Profile {
            name: name.to_string(),
            path: path.to_path_buf(),
            content,
            is_active: false, // Will be updated by update_profile_status_optimized
        })
    }
    
    /// Update the active status of profiles by comparing with default settings (optimized)
    fn update_profile_status_optimized(&mut self, profiles: &mut [Profile]) -> AppResult<()> {
        if profiles.is_empty() {
            return Ok(());
        }
        
        let default_content = match self.get_default_settings_cached() {
            Ok(content) => content,
            Err(e) => {
                log::warn!("Failed to read default settings, marking all profiles as inactive: {}", e);
                // Mark all profiles as inactive if we can't read default settings
                for profile in profiles.iter_mut() {
                    profile.is_active = false;
                }
                return Ok(());
            }
        };
        
        // Parse default content once for efficiency
        let default_json = match serde_json::from_str::<serde_json::Value>(&default_content) {
            Ok(json) => json,
            Err(e) => {
                log::warn!("Default settings contains invalid JSON, marking all profiles as inactive: {}", e);
                for profile in profiles.iter_mut() {
                    profile.is_active = false;
                }
                return Ok(());
            }
        };
        
        for profile in profiles.iter_mut() {
            profile.is_active = self.compare_configurations_optimized(&profile.content, &default_json);
        }
        
        Ok(())
    }
    
    /// Compare configuration with pre-parsed default JSON for better performance
    fn compare_configurations_optimized(&self, profile_content: &str, default_json: &serde_json::Value) -> bool {
        match serde_json::from_str::<serde_json::Value>(profile_content) {
            Ok(profile_json) => profile_json == *default_json,
            Err(e) => {
                log::warn!("Failed to parse profile content as JSON: {}", e);
                false
            }
        }
    }
    
    /// Get detailed profile status with smart comparison
    pub fn get_detailed_profile_status(&self, profile_content: &str) -> ProfileStatus {
        let default_content = match fs::read_to_string(&self.default_settings_path) {
            Ok(content) => content,
            Err(e) => {
                return ProfileStatus::Error(format!("Failed to read default settings: {}", e));
            }
        };
        
        let (default_json, profile_json) = match (
            serde_json::from_str::<serde_json::Value>(&default_content),
            serde_json::from_str::<serde_json::Value>(profile_content),
        ) {
            (Ok(default), Ok(profile)) => (default, profile),
            (Err(e), _) => {
                return ProfileStatus::Error(format!("Invalid default settings JSON: {}", e));
            }
            (_, Err(e)) => {
                return ProfileStatus::Error(format!("Invalid profile JSON: {}", e));
            }
        };
        
        // Check for full match first
        if profile_json == default_json {
            return ProfileStatus::FullMatch;
        }
        
        // Always check if only model field is different
        let matches_ignoring_model = self.compare_json_ignoring_field(&profile_json, &default_json, "model");
        if matches_ignoring_model {
            return ProfileStatus::PartialMatch;
        }
        
        ProfileStatus::NoMatch
    }
    
    /// Compare two JSON values while ignoring a specific field
    fn compare_json_ignoring_field(
        &self,
        json1: &serde_json::Value,
        json2: &serde_json::Value,
        ignore_field: &str,
    ) -> bool {
        match (json1, json2) {
            (serde_json::Value::Object(obj1), serde_json::Value::Object(obj2)) => {
                // Create modified copies without the ignored field
                let mut filtered_obj1 = obj1.clone();
                let mut filtered_obj2 = obj2.clone();
                
                filtered_obj1.remove(ignore_field);
                filtered_obj2.remove(ignore_field);
                
                filtered_obj1 == filtered_obj2
            }
            _ => json1 == json2,
        }
    }
    
    /// Read the default settings.json content
    fn read_default_settings(&self) -> AppResult<String> {
        fs::read_to_string(&self.default_settings_path)
            .map_err(|e| AppError::ConfigError(format!("Failed to read default settings: {}", e)))
    }
    
    /// Compare two configuration contents for equality
    fn compare_configurations(&self, profile_content: &str, default_content: &str) -> bool {
        // Parse both as JSON and compare to ignore formatting differences
        match (
            serde_json::from_str::<serde_json::Value>(profile_content),
            serde_json::from_str::<serde_json::Value>(default_content),
        ) {
            (Ok(profile_json), Ok(default_json)) => profile_json == default_json,
            _ => false,
        }
    }
    
    /// Get the status of all profiles with detailed comparison
    pub fn compare_profiles(&self) -> Vec<ProfileStatus> {
        let mut statuses = Vec::new();
        
        for profile in &self.profiles {
            let status = self.get_detailed_profile_status(&profile.content);
            statuses.push(status);
        }
        
        statuses
    }
    
    /// Get the status of a specific profile with detailed comparison
    pub fn get_profile_status(&self, profile_name: &str) -> ProfileStatus {
        if let Some(profile) = self.profiles.iter().find(|p| p.name == profile_name) {
            self.get_detailed_profile_status(&profile.content)
        } else {
            ProfileStatus::Error(format!("Profile '{}' not found", profile_name))
        }
    }
    
    /// Calculate CRC32 checksum for content comparison
    pub fn calculate_checksum(content: &str) -> u32 {
        crc32fast::hash(content.as_bytes())
    }
    
    /// Get file metadata for monitoring purposes
    pub fn get_file_metadata(&self, path: &Path) -> AppResult<FileMetadata> {
        let metadata = fs::metadata(path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to get file metadata: {}", e)))?;
        
        let content = fs::read_to_string(path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to read file: {}", e)))?;
        
        Ok(FileMetadata {
            modified_time: metadata.modified()
                .map_err(|e| AppError::FileSystemError(format!("Failed to get modification time: {}", e)))?,
            checksum: Self::calculate_checksum(&content),
            size: metadata.len(),
        })
    }
    
    /// Get paths of all monitored configuration files
    pub fn get_monitored_files(&self) -> Vec<PathBuf> {
        let mut files = vec![self.default_settings_path.clone()];
        
        for profile in &self.profiles {
            files.push(profile.path.clone());
        }
        
        files
    }
    
    /// Get current profiles
    pub fn get_profiles(&self) -> &[Profile] {
        &self.profiles
    }
    
    /// Switch to a specific profile configuration with enhanced error handling
    pub fn switch_profile(&mut self, profile_name: &str) -> AppResult<()> {
        log::info!("Attempting to switch to profile: {}", profile_name);
        
        // Input validation
        if profile_name.is_empty() {
            return Err(AppError::ConfigError("Profile name cannot be empty".to_string()));
        }
        
        // Find the profile
        let profile = self.profiles.iter()
            .find(|p| p.name == profile_name)
            .ok_or_else(|| AppError::ConfigError(format!("Profile '{}' not found", profile_name)))?;
        
        // Check if profile is already active
        if profile.is_active {
            log::info!("Profile '{}' is already active, no action needed", profile_name);
            return Ok(());
        }
        
        // Validate profile content before switching
        match serde_json::from_str::<serde_json::Value>(&profile.content) {
            Ok(_) => {
                log::debug!("Profile content validation passed for: {}", profile_name);
            }
            Err(e) => {
                return Err(AppError::ConfigError(
                    format!("Profile '{}' contains invalid JSON: {}", profile_name, e)
                ));
            }
        }
        
        // Pre-flight checks
        if !self.default_settings_path.exists() {
            return Err(AppError::FileSystemError(
                "Default settings file does not exist".to_string()
            ));
        }
        
        // Check if settings file is writable
        let test_write_path = self.default_settings_path.with_extension("json.write_test");
        if let Err(e) = fs::write(&test_write_path, "test") {
            return Err(AppError::FileSystemError(
                format!("Cannot write to settings directory: {}", e)
            ));
        }
        let _ = fs::remove_file(&test_write_path);
        
        // Create backup of current settings with timestamp
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let backup_path = self.default_settings_path.with_extension(format!("json.backup.{}", timestamp));
        
        if let Err(e) = self.create_backup(&backup_path) {
            return Err(AppError::FileSystemError(
                format!("Failed to create backup before switching: {}", e)
            ));
        }
        
        // Perform atomic switch operation with rollback on failure
        match self.perform_switch_atomic(&profile.content) {
            Ok(()) => {
                log::info!("Successfully switched to profile: {}", profile_name);
                
                // Clear caches since files have changed
                self.clear_cache();
                
                // Update profile status after successful switch
                if let Err(e) = self.refresh_profile_status() {
                    log::warn!("Failed to refresh profile status after switch: {}", e);
                    // Don't fail the operation since switch was successful
                }
                
                // Remove backup file on success (keep only a few recent backups)
                self.cleanup_old_backups();
                
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to switch profile, attempting rollback: {}", e);
                
                // Attempt to restore from backup
                if let Err(rollback_err) = self.restore_from_backup(&backup_path) {
                    log::error!("CRITICAL: Failed to rollback after failed switch: {}", rollback_err);
                    return Err(AppError::ConfigError(
                        format!("Profile switch failed and rollback failed. Original error: {}. Rollback error: {}", e, rollback_err)
                    ));
                }
                
                log::info!("Successfully rolled back after failed switch");
                Err(e)
            }
        }
    }
    
    /// Perform the actual configuration switch with enhanced atomic operation
    fn perform_switch_atomic(&self, new_content: &str) -> AppResult<()> {
        // Validate the new content is valid JSON with proper structure
        let json_value = serde_json::from_str::<serde_json::Value>(new_content)
            .map_err(|e| AppError::ConfigError(format!("Invalid JSON content: {}", e)))?;
        
        // Additional validation: ensure it's an object
        if !json_value.is_object() {
            return Err(AppError::ConfigError(
                "Configuration must be a JSON object".to_string()
            ));
        }
        
        // Normalize JSON formatting for consistency
        let normalized_content = serde_json::to_string_pretty(&json_value)
            .map_err(|e| AppError::ConfigError(format!("Failed to serialize JSON: {}", e)))?;
        
        // Write to temporary file first (in same directory for atomic rename)
        let temp_path = self.default_settings_path.with_extension("json.tmp");
        
        // Clean up any existing temp file
        if temp_path.exists() {
            fs::remove_file(&temp_path)
                .map_err(|e| AppError::FileSystemError(format!("Failed to clean up temp file: {}", e)))?;
        }
        
        fs::write(&temp_path, &normalized_content)
            .map_err(|e| AppError::FileSystemError(format!("Failed to write temporary file: {}", e)))?;
        
        // Verify temp file was written correctly
        let temp_verification = fs::read_to_string(&temp_path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to verify temp file: {}", e)))?;
        
        if temp_verification != normalized_content {
            let _ = fs::remove_file(&temp_path);
            return Err(AppError::FileSystemError(
                "Temp file verification failed - data corruption detected".to_string()
            ));
        }
        
        // Atomic move (rename) operation
        fs::rename(&temp_path, &self.default_settings_path)
            .map_err(|e| {
                // Clean up temp file on failure
                let _ = fs::remove_file(&temp_path);
                AppError::FileSystemError(format!("Failed to replace settings file: {}", e))
            })?;
        
        // Final verification
        let final_verification = fs::read_to_string(&self.default_settings_path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to verify final file: {}", e)))?;
        
        if final_verification != normalized_content {
            return Err(AppError::FileSystemError(
                "Final file verification failed - switch may have been corrupted".to_string()
            ));
        }
        
        log::debug!("Atomic switch operation completed successfully");
        Ok(())
    }
    
    /// Clean up old backup files (keep only the 5 most recent)
    fn cleanup_old_backups(&self) {
        let backup_pattern = format!("{}.backup.", self.default_settings_path.display());
        
        if let Ok(entries) = fs::read_dir(self.default_settings_path.parent().unwrap_or(&self.claude_dir)) {
            let mut backup_files: Vec<_> = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.file_name()
                        .to_string_lossy()
                        .starts_with(&backup_pattern)
                })
                .collect();
            
            // Sort by modification time (newest first)
            backup_files.sort_by(|a, b| {
                let time_a = a.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
                let time_b = b.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
                time_b.cmp(&time_a)
            });
            
            // Remove old backups (keep only 5 most recent)
            for old_backup in backup_files.iter().skip(5) {
                if let Err(e) = fs::remove_file(old_backup.path()) {
                    log::warn!("Failed to remove old backup {:?}: {}", old_backup.path(), e);
                }
            }
            
            if backup_files.len() > 5 {
                log::debug!("Cleaned up {} old backup files", backup_files.len() - 5);
            }
        }
    }
    
    /// Create a backup of the current settings
    fn create_backup(&self, backup_path: &Path) -> AppResult<()> {
        fs::copy(&self.default_settings_path, backup_path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to create backup: {}", e)))?;
        
        log::debug!("Created backup at: {:?}", backup_path);
        Ok(())
    }
    
    /// Restore settings from backup with enhanced error handling
    fn restore_from_backup(&self, backup_path: &Path) -> AppResult<()> {
        if !backup_path.exists() {
            return Err(AppError::FileSystemError(
                format!("Backup file does not exist: {:?}", backup_path)
            ));
        }
        
        // Verify backup file integrity before restoring
        let backup_content = fs::read_to_string(backup_path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to read backup file: {}", e)))?;
        
        // Validate backup content is valid JSON
        serde_json::from_str::<serde_json::Value>(&backup_content)
            .map_err(|e| AppError::ConfigError(format!("Backup file contains invalid JSON: {}", e)))?;
        
        // Use atomic operation for restore too
        let temp_path = self.default_settings_path.with_extension("json.restore_tmp");
        
        fs::copy(backup_path, &temp_path)
            .map_err(|e| AppError::FileSystemError(format!("Failed to copy backup to temp: {}", e)))?;
        
        fs::rename(&temp_path, &self.default_settings_path)
            .map_err(|e| {
                let _ = fs::remove_file(&temp_path);
                AppError::FileSystemError(format!("Failed to restore from backup: {}", e))
            })?;
        
        let _ = fs::remove_file(backup_path);
        log::info!("Successfully restored settings from backup");
        Ok(())
    }
    
    /// Refresh the active status of all profiles (optimized)
    pub fn refresh_profile_status(&mut self) -> AppResult<()> {
        // Create a temporary copy of profiles to avoid mutable borrow conflicts
        let mut profiles_copy = self.profiles.clone();
        self.update_profile_status_optimized(&mut profiles_copy)?;
        self.profiles = profiles_copy;
        Ok(())
    }
    
    /// Validate the integrity of a configuration switch
    pub fn validate_switch(&self, profile_name: &str) -> AppResult<bool> {
        let profile = self.profiles.iter()
            .find(|p| p.name == profile_name)
            .ok_or_else(|| AppError::ConfigError(format!("Profile '{}' not found", profile_name)))?;
        
        let current_content = self.read_default_settings()?;
        Ok(self.compare_configurations(&profile.content, &current_content))
    }
    
    /// Get Claude directory path
    pub fn get_claude_dir(&self) -> &Path {
        &self.claude_dir
    }

    /// Get all profiles information including Current
    pub fn get_all_profiles_info(&mut self) -> AppResult<Vec<ProfileInfo>> {
        // First scan for profiles to ensure we have latest data
        self.scan_profiles()?;
        
        let mut profiles_info = Vec::new();
        
        // Add Current profile first
        let _current_content = match self.get_default_settings_cached() {
            Ok(content) => content,
            Err(e) => {
                log::warn!("Failed to read current settings: {}", e);
                String::new()
            }
        };
        
        let current_metadata = self.get_file_metadata(&self.default_settings_path).ok();
        
        profiles_info.push(ProfileInfo {
            id: "current".to_string(),
            display_name: "Current".to_string(),
            file_path: self.default_settings_path.to_string_lossy().to_string(),
            is_default: true,
            last_modified: current_metadata.as_ref()
                .map(|m| m.modified_time)
                .unwrap_or(SystemTime::UNIX_EPOCH),
            file_size: current_metadata.as_ref()
                .map(|m| m.size)
                .unwrap_or(0),
        });
        
        // Add all other profiles
        for profile in &self.profiles {
            profiles_info.push(ProfileInfo {
                id: profile.name.clone(),
                display_name: profile.name.clone(),
                file_path: profile.path.to_string_lossy().to_string(),
                is_default: false,
                last_modified: self.get_file_metadata(&profile.path)
                    .map(|m| m.modified_time)
                    .unwrap_or(SystemTime::UNIX_EPOCH),
                file_size: self.get_file_metadata(&profile.path)
                    .map(|m| m.size)
                    .unwrap_or(0),
            });
        }
        
        Ok(profiles_info)
    }

    /// Read profile content by profile ID
    pub fn read_profile_content(&mut self, profile_id: &str) -> AppResult<String> {
        if profile_id == "current" {
            return self.get_default_settings_cached();
        }
        
        // Find the profile and clone the path to avoid borrowing conflicts
        let profile_path = self.profiles.iter()
            .find(|p| p.name == profile_id)
            .map(|p| p.path.clone())
            .ok_or_else(|| AppError::ConfigError(format!("Profile '{}' not found", profile_id)))?;
        
        self.get_file_content_cached(&profile_path)
    }

    /// Save profile content by profile ID
    pub fn save_profile_content(&mut self, profile_id: &str, content: &str) -> AppResult<()> {
        // Validate JSON content first
        let validation_result = self.validate_json_content(content)?;
        if !validation_result.is_valid {
            return Err(AppError::ConfigError("JSON validation failed".to_string()));
        }
        
        if profile_id == "current" {
            return self.save_current_settings(content);
        }
        
        // Find the profile and clone the path to avoid borrowing conflicts
        let profile_path = self.profiles.iter()
            .find(|p| p.name == profile_id)
            .map(|p| p.path.clone())
            .ok_or_else(|| AppError::ConfigError(format!("Profile '{}' not found", profile_id)))?;
        
        // Save to profile file
        self.save_to_file(&profile_path, content)?;
        
        // Update cache
        self.profile_cache.remove(&profile_path);
        
        Ok(())
    }

    /// Create a new profile with given name and content
    pub fn create_profile(&mut self, profile_name: &str, content: &str) -> AppResult<String> {
        // Validate profile name
        self.validate_profile_name(profile_name)?;
        
        // Validate JSON content
        let validation_result = self.validate_json_content(content)?;
        if !validation_result.is_valid {
            return Err(AppError::ConfigError("JSON validation failed".to_string()));
        }
        
        // Check if profile already exists
        if self.profiles.iter().any(|p| p.name == profile_name) {
            return Err(AppError::ConfigError(format!("Profile '{}' already exists", profile_name)));
        }
        
        // Create profile file path
        let profile_file_name = format!("{}.settings.json", profile_name);
        let profile_path = self.claude_dir.join(&profile_file_name);
        
        // Save to file
        self.save_to_file(&profile_path, content)?;
        
        // Add to profiles list
        let new_profile = Profile {
            name: profile_name.to_string(),
            path: profile_path.clone(),
            content: content.to_string(),
            is_active: false,
        };
        self.profiles.push(new_profile);
        
        // Refresh profile status
        self.refresh_profile_status()?;
        
        Ok(profile_path.to_string_lossy().to_string())
    }

    /// Validate profile name
    pub fn validate_profile_name(&self, name: &str) -> AppResult<()> {
        if name.is_empty() {
            return Err(AppError::ConfigError("Profile name cannot be empty".to_string()));
        }
        
        if name.len() > 255 {
            return Err(AppError::ConfigError("Profile name too long (max 255 characters)".to_string()));
        }
        
        // Check for invalid characters
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];
        if name.chars().any(|c| invalid_chars.contains(&c)) {
            return Err(AppError::ConfigError(
                "Profile name contains invalid characters. Avoid: / \\ : * ? \" < > |".to_string()
            ));
        }
        
        // Check for reserved names
        let reserved_names = ["current", "settings", "backup"];
        if reserved_names.contains(&name.to_lowercase().as_str()) {
            return Err(AppError::ConfigError(
                format!("'{}' is a reserved name and cannot be used", name)
            ));
        }
        
        Ok(())
    }

    /// Validate JSON content using extensible validation framework
    pub fn validate_json_content(&self, content: &str) -> AppResult<ValidationResult> {
        self.validator.validate(content)
    }

    /// Get the names of all active validation rules
    pub fn get_validation_rules(&self) -> Vec<&'static str> {
        self.validator.get_rule_names()
    }

    /// Add custom validation rule (for future extensions)
    pub fn add_validation_rule(&mut self, rule: Box<dyn crate::validation::ValidationRule>) {
        self.validator.add_rule(rule);
    }

    /// Save current settings (settings.json)
    fn save_current_settings(&self, content: &str) -> AppResult<()> {
        self.save_to_file(&self.default_settings_path, content)
    }

    /// Save content to file with atomic operation
    fn save_to_file(&self, path: &Path, content: &str) -> AppResult<()> {
        // Validate and normalize JSON
        let json_value = serde_json::from_str::<serde_json::Value>(content)
            .map_err(|e| AppError::ConfigError(format!("Invalid JSON content: {}", e)))?;
        
        let normalized_content = serde_json::to_string_pretty(&json_value)
            .map_err(|e| AppError::ConfigError(format!("Failed to serialize JSON: {}", e)))?;
        
        // Write to temporary file first
        let temp_path = path.with_extension("tmp");
        
        // Clean up any existing temp file
        if temp_path.exists() {
            fs::remove_file(&temp_path)
                .map_err(|e| AppError::FileSystemError(format!("Failed to clean up temp file: {}", e)))?;
        }
        
        fs::write(&temp_path, &normalized_content)
            .map_err(|e| AppError::FileSystemError(format!("Failed to write temporary file: {}", e)))?;
        
        // Atomic move
        fs::rename(&temp_path, path)
            .map_err(|e| {
                let _ = fs::remove_file(&temp_path);
                AppError::FileSystemError(format!("Failed to save file: {}", e))
            })?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    fn create_test_config_service() -> (ConfigService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let claude_dir = temp_dir.path().to_path_buf();
        
        // Create default settings.json
        let settings_content = r#"{"theme": "dark", "language": "en"}"#;
        fs::write(claude_dir.join("settings.json"), settings_content).unwrap();
        
        let service = ConfigService::new(claude_dir);
        (service, temp_dir)
    }
    
    #[test]
    fn test_scan_profiles_empty_directory() {
        let (mut service, _temp_dir) = create_test_config_service();
        
        let profiles = service.scan_profiles().unwrap();
        assert!(profiles.is_empty());
    }
    
    #[test]
    fn test_scan_profiles_with_profiles() {
        let (mut service, temp_dir) = create_test_config_service();
        
        // Create test profile files
        let profile1_content = r#"{"theme": "light", "language": "en"}"#;
        let profile2_content = r#"{"theme": "dark", "language": "en"}"#; // Same as default
        
        fs::write(temp_dir.path().join("work.settings.json"), profile1_content).unwrap();
        fs::write(temp_dir.path().join("personal.settings.json"), profile2_content).unwrap();
        
        let profiles = service.scan_profiles().unwrap();
        assert_eq!(profiles.len(), 2);
        
        // Check that personal profile is active (matches default)
        let personal_profile = profiles.iter().find(|p| p.name == "personal").unwrap();
        assert!(personal_profile.is_active);
        
        let work_profile = profiles.iter().find(|p| p.name == "work").unwrap();
        assert!(!work_profile.is_active);
    }
    
    #[test]
    fn test_compare_configurations() {
        let (service, _temp_dir) = create_test_config_service();
        
        let config1 = r#"{"theme": "dark", "language": "en"}"#;
        let config2 = r#"{"language": "en", "theme": "dark"}"#; // Same content, different order
        let config3 = r#"{"theme": "light", "language": "en"}"#;
        
        assert!(service.compare_configurations(config1, config2));
        assert!(!service.compare_configurations(config1, config3));
    }
    
    #[test]
    fn test_calculate_checksum() {
        let content1 = "test content";
        let content2 = "test content";
        let content3 = "different content";
        
        assert_eq!(
            ConfigService::calculate_checksum(content1),
            ConfigService::calculate_checksum(content2)
        );
        assert_ne!(
            ConfigService::calculate_checksum(content1),
            ConfigService::calculate_checksum(content3)
        );
    }
    
    #[test]
    fn test_switch_profile_success() {
        let (mut service, temp_dir) = create_test_config_service();
        
        // Create a test profile
        let profile_content = r#"{"theme": "light", "language": "fr"}"#;
        fs::write(temp_dir.path().join("test.settings.json"), profile_content).unwrap();
        
        // Scan profiles
        service.scan_profiles().unwrap();
        
        // Switch to test profile
        let result = service.switch_profile("test");
        assert!(result.is_ok());
        
        // Verify the switch was successful
        let validation = service.validate_switch("test").unwrap();
        assert!(validation);
    }
    
    #[test]
    fn test_switch_profile_already_active() {
        let (mut service, temp_dir) = create_test_config_service();
        
        // Create a profile with same content as default
        let profile_content = r#"{"theme": "dark", "language": "en"}"#;
        fs::write(temp_dir.path().join("same.settings.json"), profile_content).unwrap();
        
        // Scan profiles
        service.scan_profiles().unwrap();
        
        // Switch to already active profile
        let result = service.switch_profile("same");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_switch_profile_not_found() {
        let (mut service, _temp_dir) = create_test_config_service();
        
        let result = service.switch_profile("nonexistent");
        assert!(result.is_err());
        
        if let Err(AppError::ConfigError(msg)) = result {
            assert!(msg.contains("not found"));
        } else {
            panic!("Expected ConfigError");
        }
    }
    
    #[test]
    fn test_backup_and_restore() {
        let (service, temp_dir) = create_test_config_service();
        let backup_path = temp_dir.path().join("test.backup");
        
        // Create backup
        service.create_backup(&backup_path).unwrap();
        assert!(backup_path.exists());
        
        // Modify original file
        fs::write(&service.default_settings_path, "modified content").unwrap();
        
        // Restore from backup
        service.restore_from_backup(&backup_path).unwrap();
        
        // Verify restoration
        let restored_content = fs::read_to_string(&service.default_settings_path).unwrap();
        assert_eq!(restored_content, r#"{"theme": "dark", "language": "en"}"#);
        assert!(!backup_path.exists()); // Backup should be removed
    }
}