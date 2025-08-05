// Settings service for user configuration management
use crate::{AppError, AppResult, UserSettings};
use std::path::{Path, PathBuf};
use std::fs;

pub struct SettingsService {
    settings_file_path: PathBuf,
    current_settings: UserSettings,
}

impl SettingsService {
    /// Create a new settings service
    pub fn new() -> AppResult<Self> {
        let settings_dir = Self::get_settings_directory()?;
        
        // Ensure settings directory exists
        if !settings_dir.exists() {
            fs::create_dir_all(&settings_dir)
                .map_err(|e| AppError::SettingsError(format!("Failed to create settings directory: {}", e)))?;
        }
        
        let settings_file_path = settings_dir.join("cccs_settings.json");
        
        let mut service = Self {
            settings_file_path,
            current_settings: UserSettings::default(),
        };
        
        // Load existing settings or create default
        service.load_or_create_settings()?;
        Ok(service)
    }
    
    /// Create a settings service with default values (fallback)
    pub fn with_defaults() -> Self {
        let settings_file_path = dirs::config_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join("cccs")
            .join("settings.json");
        
        Self {
            settings_file_path,
            current_settings: UserSettings::default(),
        }
    }
    
    /// Get the platform-specific settings directory
    fn get_settings_directory() -> AppResult<PathBuf> {
        dirs::config_dir()
            .map(|dir| dir.join("cccs"))
            .ok_or_else(|| AppError::SettingsError("Failed to get config directory".to_string()))
    }
    
    /// Load settings from file or create default settings
    fn load_or_create_settings(&mut self) -> AppResult<()> {
        if self.settings_file_path.exists() {
            self.load_settings()?;
        } else {
            self.save_settings(&UserSettings::default())?;
        }
        Ok(())
    }
    
    /// Load settings from the settings file
    pub fn load_settings(&mut self) -> AppResult<UserSettings> {
        log::info!("Loading settings from: {:?}", self.settings_file_path);
        
        let content = fs::read_to_string(&self.settings_file_path)
            .map_err(|e| AppError::SettingsError(format!("Failed to read settings file: {}", e)))?;
        
        let settings: UserSettings = serde_json::from_str(&content)
            .map_err(|e| AppError::SettingsError(format!("Failed to parse settings JSON: {}", e)))?;
        
        // Validate settings
        Self::validate_settings(&settings)?;
        
        self.current_settings = settings.clone();
        log::info!("Settings loaded successfully");
        
        Ok(settings)
    }
    
    /// Save settings to the settings file
    pub fn save_settings(&self, settings: &UserSettings) -> AppResult<()> {
        log::info!("Saving settings to: {:?}", self.settings_file_path);
        
        // Validate settings before saving
        Self::validate_settings(settings)?;
        
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| AppError::SettingsError(format!("Failed to serialize settings: {}", e)))?;
        
        // Write to temporary file first for atomic operation
        let temp_path = self.settings_file_path.with_extension("json.tmp");
        fs::write(&temp_path, content)
            .map_err(|e| AppError::SettingsError(format!("Failed to write temporary settings file: {}", e)))?;
        
        // Atomic move
        fs::rename(&temp_path, &self.settings_file_path)
            .map_err(|e| AppError::SettingsError(format!("Failed to save settings file: {}", e)))?;
        
        log::info!("Settings saved successfully");
        Ok(())
    }
    
    /// Validate settings values
    fn validate_settings(settings: &UserSettings) -> AppResult<()> {
        Self::validate_monitor_interval(settings.monitor_interval_minutes)?;
        
        // Validate language if specified
        if let Some(ref language) = settings.language {
            if !Self::is_supported_language(language) {
                return Err(AppError::SettingsError(
                    format!("Unsupported language: {}", language)
                ));
            }
        }
        
        // Validate ignored fields
        crate::UserSettings::validate_ignored_fields(&settings.ignored_fields)
            .map_err(|e| AppError::SettingsError(format!("Invalid ignored fields in settings: {}", e)))?;
        
        Ok(())
    }
    
    /// Validate monitor interval (1-60 minutes)
    pub fn validate_monitor_interval(minutes: u64) -> AppResult<()> {
        if !(1..=60).contains(&minutes) {
            return Err(AppError::SettingsError(
                format!("Invalid monitor interval: {} minutes. Must be between 1 and 60.", minutes)
            ));
        }
        Ok(())
    }
    
    /// Check if a language is supported
    fn is_supported_language(language: &str) -> bool {
        matches!(language, "en" | "zh" | "zh-CN" | "zh-TW")
    }
    
    /// Update current settings and save
    pub fn update_settings(&mut self, new_settings: UserSettings) -> AppResult<()> {
        log::info!("Updating settings");
        
        self.save_settings(&new_settings)?;
        self.current_settings = new_settings;
        
        Ok(())
    }
    
    /// Update monitor interval
    pub fn update_monitor_interval(&mut self, minutes: u64) -> AppResult<()> {
        Self::validate_monitor_interval(minutes)?;
        
        self.current_settings.monitor_interval_minutes = minutes;
        self.save_settings(&self.current_settings)?;
        
        log::info!("Monitor interval updated to {} minutes", minutes);
        Ok(())
    }
    
    /// Update auto start monitoring setting
    pub fn update_auto_start_monitoring(&mut self, enabled: bool) -> AppResult<()> {
        self.current_settings.auto_start_monitoring = enabled;
        self.save_settings(&self.current_settings)?;
        
        log::info!("Auto start monitoring set to: {}", enabled);
        Ok(())
    }
    
    /// Update language setting
    pub fn update_language(&mut self, language: Option<String>) -> AppResult<()> {
        if let Some(ref lang) = language {
            if !Self::is_supported_language(lang) {
                return Err(AppError::SettingsError(
                    format!("Unsupported language: {}", lang)
                ));
            }
        }
        
        self.current_settings.language = language;
        self.save_settings(&self.current_settings)?;
        
        log::info!("Language setting updated");
        Ok(())
    }
    
    /// Update notifications setting
    pub fn update_show_notifications(&mut self, enabled: bool) -> AppResult<()> {
        self.current_settings.show_notifications = enabled;
        self.save_settings(&self.current_settings)?;
        
        log::info!("Show notifications set to: {}", enabled);
        Ok(())
    }
    
    /// Get current settings
    pub fn get_current_settings(&self) -> &UserSettings {
        &self.current_settings
    }
    
    /// Get ignored fields list
    pub fn get_ignored_fields(&self) -> &[String] {
        &self.current_settings.ignored_fields
    }
    
    /// Update ignored fields list
    pub fn update_ignored_fields(&mut self, fields: Vec<String>) -> AppResult<()> {
        // 验证字段列表
        crate::UserSettings::validate_ignored_fields(&fields)
            .map_err(|e| AppError::SettingsError(format!("Invalid ignored fields: {}", e)))?;
        
        // 标准化字段列表
        let normalized_fields = crate::UserSettings::normalize_ignored_fields(fields);
        
        log::info!("Updating ignored fields: {:?}", normalized_fields);
        
        // 更新设置
        self.current_settings.ignored_fields = normalized_fields;
        
        // 保存到文件
        self.save_settings(&self.current_settings)?;
        
        log::info!("Ignored fields updated successfully");
        Ok(())
    }
    
    /// Get default ignored fields
    pub fn get_default_ignored_fields() -> Vec<String> {
        crate::UserSettings::get_default_ignored_fields()
    }
    
    /// Reset ignored fields to default
    pub fn reset_ignored_fields_to_default(&mut self) -> AppResult<()> {
        let default_fields = Self::get_default_ignored_fields();
        log::info!("Resetting ignored fields to default: {:?}", default_fields);
        
        self.current_settings.ignored_fields = default_fields;
        self.save_settings(&self.current_settings)?;
        
        log::info!("Ignored fields reset to default successfully");
        Ok(())
    }
    
    /// Get settings file path
    pub fn get_settings_file_path(&self) -> &Path {
        &self.settings_file_path
    }
    
    /// Reset settings to default
    pub fn reset_to_defaults(&mut self) -> AppResult<()> {
        log::info!("Resetting settings to defaults");
        
        let default_settings = UserSettings::default();
        self.save_settings(&default_settings)?;
        self.current_settings = default_settings;
        
        Ok(())
    }
    
    /// Export settings to a specified file
    pub fn export_settings(&self, export_path: &Path) -> AppResult<()> {
        log::info!("Exporting settings to: {:?}", export_path);
        
        let content = serde_json::to_string_pretty(&self.current_settings)
            .map_err(|e| AppError::SettingsError(format!("Failed to serialize settings for export: {}", e)))?;
        
        fs::write(export_path, content)
            .map_err(|e| AppError::SettingsError(format!("Failed to export settings: {}", e)))?;
        
        log::info!("Settings exported successfully");
        Ok(())
    }
    
    /// Import settings from a specified file
    pub fn import_settings(&mut self, import_path: &Path) -> AppResult<()> {
        log::info!("Importing settings from: {:?}", import_path);
        
        let content = fs::read_to_string(import_path)
            .map_err(|e| AppError::SettingsError(format!("Failed to read import file: {}", e)))?;
        
        let imported_settings: UserSettings = serde_json::from_str(&content)
            .map_err(|e| AppError::SettingsError(format!("Failed to parse imported settings: {}", e)))?;
        
        // Validate imported settings
        Self::validate_settings(&imported_settings)?;
        
        self.update_settings(imported_settings)?;
        
        log::info!("Settings imported successfully");
        Ok(())
    }
    
    /// Create backup of current settings
    pub fn create_backup(&self) -> AppResult<PathBuf> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AppError::SettingsError(format!("Failed to get timestamp: {}", e)))?
            .as_secs();
        
        let backup_filename = format!("cccs_settings_backup_{}.json", timestamp);
        let backup_path = self.settings_file_path
            .parent()
            .unwrap()
            .join(backup_filename);
        
        self.export_settings(&backup_path)?;
        
        log::info!("Settings backup created: {:?}", backup_path);
        Ok(backup_path)
    }
}

// Tauri commands for settings management
#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, std::sync::Mutex<SettingsService>>) -> Result<UserSettings, String> {
    let service = state.lock().map_err(|e| format!("Failed to lock settings service: {}", e))?;
    Ok(service.get_current_settings().clone())
}

#[tauri::command]
pub async fn update_monitor_interval(
    minutes: u64,
    state: tauri::State<'_, std::sync::Mutex<SettingsService>>,
) -> Result<(), String> {
    let mut service = state.lock().map_err(|e| format!("Failed to lock settings service: {}", e))?;
    service.update_monitor_interval(minutes)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_auto_start_monitoring(
    enabled: bool,
    state: tauri::State<'_, std::sync::Mutex<SettingsService>>,
) -> Result<(), String> {
    let mut service = state.lock().map_err(|e| format!("Failed to lock settings service: {}", e))?;
    service.update_auto_start_monitoring(enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_language(
    language: Option<String>,
    state: tauri::State<'_, std::sync::Mutex<SettingsService>>,
) -> Result<(), String> {
    let mut service = state.lock().map_err(|e| format!("Failed to lock settings service: {}", e))?;
    service.update_language(language)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_show_notifications(
    enabled: bool,
    state: tauri::State<'_, std::sync::Mutex<SettingsService>>,
) -> Result<(), String> {
    let mut service = state.lock().map_err(|e| format!("Failed to lock settings service: {}", e))?;
    service.update_show_notifications(enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_settings_to_defaults(
    state: tauri::State<'_, std::sync::Mutex<SettingsService>>,
) -> Result<(), String> {
    let mut service = state.lock().map_err(|e| format!("Failed to lock settings service: {}", e))?;
    service.reset_to_defaults()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_ignored_fields(
    state: tauri::State<'_, std::sync::Mutex<SettingsService>>,
) -> Result<Vec<String>, String> {
    let service = state.lock().map_err(|e| format!("Failed to lock settings service: {}", e))?;
    Ok(service.get_ignored_fields().to_vec())
}

#[tauri::command]
pub async fn update_ignored_fields(
    fields: Vec<String>,
    state: tauri::State<'_, std::sync::Mutex<SettingsService>>,
) -> Result<(), String> {
    let mut service = state.lock().map_err(|e| format!("Failed to lock settings service: {}", e))?;
    service.update_ignored_fields(fields)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_default_ignored_fields() -> Result<Vec<String>, String> {
    Ok(SettingsService::get_default_ignored_fields())
}

#[tauri::command]
pub async fn reset_ignored_fields_to_default(
    state: tauri::State<'_, std::sync::Mutex<SettingsService>>,
) -> Result<(), String> {
    let mut service = state.lock().map_err(|e| format!("Failed to lock settings service: {}", e))?;
    service.reset_ignored_fields_to_default()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::env;
    
    fn create_test_settings_service() -> (SettingsService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        
        // Override the config directory for testing
        env::set_var("HOME", temp_dir.path());
        
        let service = SettingsService::new().unwrap();
        (service, temp_dir)
    }
    
    #[test]
    fn test_settings_service_creation() {
        let (_service, _temp_dir) = create_test_settings_service();
        // If we get here without panic, the service was created successfully
    }
    
    #[test]
    fn test_validate_monitor_interval() {
        assert!(SettingsService::validate_monitor_interval(1).is_ok());
        assert!(SettingsService::validate_monitor_interval(30).is_ok());
        assert!(SettingsService::validate_monitor_interval(60).is_ok());
        
        assert!(SettingsService::validate_monitor_interval(0).is_err());
        assert!(SettingsService::validate_monitor_interval(61).is_err());
    }
    
    #[test]
    fn test_is_supported_language() {
        assert!(SettingsService::is_supported_language("en"));
        assert!(SettingsService::is_supported_language("zh"));
        assert!(SettingsService::is_supported_language("zh-CN"));
        assert!(SettingsService::is_supported_language("zh-TW"));
        
        assert!(!SettingsService::is_supported_language("fr"));
        assert!(!SettingsService::is_supported_language("invalid"));
    }
    
    #[test]
    fn test_update_monitor_interval() {
        let (mut service, _temp_dir) = create_test_settings_service();
        
        assert!(service.update_monitor_interval(15).is_ok());
        assert_eq!(service.get_current_settings().monitor_interval_minutes, 15);
        
        assert!(service.update_monitor_interval(0).is_err());
        // Value should remain unchanged after error
        assert_eq!(service.get_current_settings().monitor_interval_minutes, 15);
    }
    
    #[test]
    fn test_update_language() {
        let (mut service, _temp_dir) = create_test_settings_service();
        
        assert!(service.update_language(Some("zh".to_string())).is_ok());
        assert_eq!(service.get_current_settings().language, Some("zh".to_string()));
        
        assert!(service.update_language(None).is_ok());
        assert_eq!(service.get_current_settings().language, None);
        
        assert!(service.update_language(Some("invalid".to_string())).is_err());
    }
    
    #[test]
    fn test_settings_persistence() {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("HOME", temp_dir.path());
        
        // Create service and update settings
        {
            let mut service = SettingsService::new().unwrap();
            service.update_monitor_interval(25).unwrap();
            service.update_auto_start_monitoring(false).unwrap();
        }
        
        // Create new service and verify settings persisted
        {
            let service = SettingsService::new().unwrap();
            let settings = service.get_current_settings();
            assert_eq!(settings.monitor_interval_minutes, 25);
            assert!(!settings.auto_start_monitoring);
        }
    }
    
    #[test]
    fn test_export_import_settings() {
        let (service, temp_dir) = create_test_settings_service();
        
        let export_path = temp_dir.path().join("exported_settings.json");
        
        // Export settings
        assert!(service.export_settings(&export_path).is_ok());
        assert!(export_path.exists());
        
        // Create new service and import
        let (mut new_service, _) = create_test_settings_service();
        new_service.update_monitor_interval(30).unwrap(); // Change to different value
        
        assert!(new_service.import_settings(&export_path).is_ok());
        
        // Settings should match original
        assert_eq!(
            new_service.get_current_settings().monitor_interval_minutes,
            service.get_current_settings().monitor_interval_minutes
        );
    }
    
    #[test]
    fn test_create_backup() {
        let (service, _temp_dir) = create_test_settings_service();
        
        let backup_path = service.create_backup().unwrap();
        assert!(backup_path.exists());
        assert!(backup_path.file_name().unwrap().to_str().unwrap().starts_with("cccs_settings_backup_"));
    }
}