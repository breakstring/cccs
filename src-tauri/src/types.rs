// CCCS Types definitions
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub id: String,
    pub display_name: String,
    pub file_path: String,
    pub is_default: bool,
    pub last_modified: SystemTime,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub error_type: String,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub modified_time: SystemTime,
    pub checksum: u32,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub monitor_interval_minutes: u64,
    pub auto_start_monitoring: bool,
    pub language: Option<String>,
    pub show_notifications: bool,
    #[serde(default = "UserSettings::get_default_ignored_fields")]
    pub ignored_fields: Vec<String>, // 配置比较时要忽略的字段列表
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            monitor_interval_minutes: 5,
            auto_start_monitoring: true,
            language: None,
            show_notifications: true,
            ignored_fields: Self::get_default_ignored_fields(),
        }
    }
}

impl UserSettings {
    /// 获取默认的忽略字段列表
    pub fn get_default_ignored_fields() -> Vec<String> {
        vec![
            "model".to_string(),
            "feedbackSurveyState".to_string(),
        ]
    }
    
    /// 验证忽略字段列表的有效性
    pub fn validate_ignored_fields(fields: &[String]) -> Result<(), String> {
        for field in fields {
            let field = field.trim();
            if field.is_empty() {
                return Err("字段名不能为空".to_string());
            }
            
            // 检查字段名是否包含无效字符
            if field.contains(|c: char| c.is_whitespace() || "{}[]\"'\\".contains(c)) {
                return Err(format!("字段名 '{}' 包含无效字符", field));
            }
            
            // 检查字段名长度
            if field.len() > 100 {
                return Err(format!("字段名 '{}' 过长（最大100字符）", field));
            }
        }
        
        Ok(())
    }
    
    /// 清理和标准化忽略字段列表
    pub fn normalize_ignored_fields(fields: Vec<String>) -> Vec<String> {
        fields
            .into_iter()
            .map(|field| field.trim().to_string())
            .filter(|field| !field.is_empty())
            .collect::<std::collections::HashSet<_>>() // 去重
            .into_iter()
            .collect()
    }
}

#[derive(Debug, PartialEq)]
pub enum ProfileStatus {
    FullMatch,      // 完全匹配 ✅
    PartialMatch,   // 部分匹配（忽略model字段后匹配）🔄
    NoMatch,        // 不匹配 ❌
    Error(String),  // 错误状态
}

#[derive(Debug)]
pub struct ConfigFileChange {
    pub file_path: PathBuf,
    pub change_type: ChangeType,
}

#[derive(Debug)]
pub enum ChangeType {
    Modified,
    Created,
    Deleted,
}

// Performance monitoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStats {
    pub monitored_files_count: usize,
    pub cached_metadata_count: usize,
    pub current_error_count: u32,
    pub is_running: bool,
    pub interval_minutes: u64,
    pub cache_size_limit: usize,
    pub max_scan_errors: u32,
}

// Performance test configuration
#[derive(Debug, Clone)]
pub struct PerformanceTestConfig {
    pub test_duration_seconds: u64,
    pub file_count: usize,
    pub file_size_bytes: usize,
    pub modification_frequency_seconds: u64,
}

impl Default for PerformanceTestConfig {
    fn default() -> Self {
        Self {
            test_duration_seconds: 60,
            file_count: 10,
            file_size_bytes: 1024,
            modification_frequency_seconds: 5,
        }
    }
}