// CCCS - Claude Code Configuration Switcher
// Core modules
mod app;
mod claude_detector;
mod config_service;
mod error;
mod i18n_service;
mod monitor_service;
mod settings_service;
mod tray_service;
mod types;
mod validation;

// Performance testing module (only in debug builds)
#[cfg(debug_assertions)]
pub mod performance_tests;

// Re-exports for public API
pub use error::AppError;
pub use types::*;

pub type AppResult<T> = Result<T, AppError>;

use app::App;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
struct ProfilesInfo {
    claude_directory: String,
    profiles_count: usize,
    monitor_status: String,
}

#[tauri::command]
async fn get_profiles_info(
    app_state: tauri::State<'_, Arc<Mutex<App>>>,
) -> Result<ProfilesInfo, String> {
    log::info!("get_profiles_info called");

    let app = match app_state.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock app state: {}", e);
            return Err("Failed to access application state".to_string());
        }
    };

    let config_service = app.get_config_service();
    let config = match config_service.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock config service: {}", e);
            return Err("Failed to access configuration service".to_string());
        }
    };

    let profiles = config.get_profiles();
    let claude_dir = config.get_claude_dir();

    log::info!(
        "Returning profiles info: {} profiles found in {}",
        profiles.len(),
        claude_dir.display()
    );

    Ok(ProfilesInfo {
        claude_directory: claude_dir.to_string_lossy().to_string(),
        profiles_count: profiles.len(),
        monitor_status: "inactive".to_string(),
    })
}

#[tauri::command]
async fn get_profiles_list(
    app_state: tauri::State<'_, Arc<Mutex<App>>>,
) -> Result<Vec<ProfileInfo>, String> {
    log::info!("get_profiles_list called");

    let app = match app_state.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock app state: {}", e);
            return Err("Failed to access application state".to_string());
        }
    };

    let config_service = app.get_config_service();
    let mut config = match config_service.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock config service: {}", e);
            return Err("Failed to access configuration service".to_string());
        }
    };

    match config.get_all_profiles_info() {
        Ok(profiles) => {
            log::info!("Returning {} profiles", profiles.len());
            Ok(profiles)
        }
        Err(e) => {
            log::error!("Failed to get profiles list: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn get_profile_status(
    profile_id: String,
    app_state: tauri::State<'_, Arc<Mutex<App>>>,
    settings_state: tauri::State<'_, std::sync::Mutex<settings_service::SettingsService>>,
) -> Result<String, String> {
    log::debug!("get_profile_status called for profile: {}", profile_id);

    if profile_id == "current" {
        return Ok("".to_string()); // Current profile doesn't have status icon
    }

    let app = match app_state.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock app state: {}", e);
            return Err("Failed to access application state".to_string());
        }
    };

    let config_service = app.get_config_service();
    let mut config = match config_service.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock config service: {}", e);
            return Err("Failed to access configuration service".to_string());
        }
    };

    // Get ignored fields from settings
    let ignored_fields = match settings_state.lock() {
        Ok(settings) => settings.get_ignored_fields().to_vec(),
        Err(e) => {
            log::warn!("Failed to get ignored fields from settings: {}, using defaults", e);
            crate::UserSettings::get_default_ignored_fields()
        }
    };

    match config.read_profile_content(&profile_id) {
        Ok(content) => {
            let status = config.get_detailed_profile_status_with_ignored_fields(&content, Some(&ignored_fields));
            let icon = match status {
                crate::ProfileStatus::FullMatch => "✅",
                crate::ProfileStatus::PartialMatch => "🔄",
                crate::ProfileStatus::Error(_) => "❌",
                crate::ProfileStatus::NoMatch => "",
            };
            Ok(icon.to_string())
        }
        Err(e) => {
            log::error!("Failed to read profile content: {}", e);
            Ok("❌".to_string())
        }
    }
}

#[tauri::command]
async fn load_profile_content(
    profile_id: String,
    app_state: tauri::State<'_, Arc<Mutex<App>>>,
) -> Result<String, String> {
    log::info!("load_profile_content called for profile: {}", profile_id);

    // Use blocking lock for load_profile_content to avoid race conditions
    // This is critical for the Current profile loading issue
    let app = app_state.lock().unwrap_or_else(|poisoned| {
        log::warn!("App state lock was poisoned, recovering");
        poisoned.into_inner()
    });

    let config_service = app.get_config_service();
    let mut config = config_service.lock().unwrap_or_else(|poisoned| {
        log::warn!("Config service lock was poisoned, recovering");
        poisoned.into_inner()
    });

    match config.read_profile_content(&profile_id) {
        Ok(content) => {
            log::info!("Successfully loaded content for profile: {}", profile_id);
            Ok(content)
        }
        Err(e) => {
            log::error!("Failed to load profile content: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn save_profile(
    profile_id: String,
    content: String,
    app_state: tauri::State<'_, Arc<Mutex<App>>>,
) -> Result<(), String> {
    log::info!("save_profile called for profile: {}", profile_id);

    let app = match app_state.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock app state: {}", e);
            return Err("Failed to access application state".to_string());
        }
    };

    let config_service = app.get_config_service();
    let mut config = match config_service.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock config service: {}", e);
            return Err("Failed to access configuration service".to_string());
        }
    };

    match config.save_profile_content(&profile_id, &content) {
        Ok(()) => {
            log::info!("Successfully saved profile: {}", profile_id);
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to save profile: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn create_new_profile(
    profile_name: String,
    content: String,
    app_state: tauri::State<'_, Arc<Mutex<App>>>,
) -> Result<String, String> {
    log::info!("create_new_profile called for profile: {}", profile_name);

    let app = match app_state.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock app state: {}", e);
            return Err("Failed to access application state".to_string());
        }
    };

    let config_service = app.get_config_service();
    let mut config = match config_service.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock config service: {}", e);
            return Err("Failed to access configuration service".to_string());
        }
    };

    match config.create_profile(&profile_name, &content) {
        Ok(path) => {
            log::info!("Successfully created profile: {} at {}", profile_name, path);
            Ok(path)
        }
        Err(e) => {
            log::error!("Failed to create profile: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn delete_profile(
    profile_id: String,
    app_state: tauri::State<'_, Arc<Mutex<App>>>,
) -> Result<(), String> {
    log::info!("delete_profile called for profile: {}", profile_id);

    let app = match app_state.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock app state: {}", e);
            return Err("Failed to access application state".to_string());
        }
    };

    let config_service = app.get_config_service();
    let mut config = match config_service.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock config service: {}", e);
            return Err("Failed to access configuration service".to_string());
        }
    };

    match config.delete_profile(&profile_id) {
        Ok(()) => {
            log::info!("Successfully deleted profile: {}", profile_id);
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to delete profile: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn validate_json_content(
    content: String,
    app_state: tauri::State<'_, Arc<Mutex<App>>>,
) -> Result<ValidationResult, String> {
    log::debug!("validate_json_content called");

    let app = match app_state.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock app state: {}", e);
            return Err("Failed to access application state".to_string());
        }
    };

    let config_service = app.get_config_service();
    let config = match config_service.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock config service: {}", e);
            return Err("Failed to access configuration service".to_string());
        }
    };

    match config.validate_json_content(&content) {
        Ok(result) => {
            log::debug!("JSON validation result: valid={}", result.is_valid);
            Ok(result)
        }
        Err(e) => {
            log::error!("Failed to validate JSON content: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn close_settings_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("settings") {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            settings_service::get_settings,
            settings_service::update_monitor_interval,
            settings_service::update_auto_start_monitoring,
            settings_service::update_language,
            settings_service::update_show_notifications,
            settings_service::reset_settings_to_defaults,
            settings_service::get_ignored_fields,
            settings_service::update_ignored_fields,
            settings_service::get_default_ignored_fields,
            settings_service::reset_ignored_fields_to_default,
            i18n_service::get_current_locale,
            i18n_service::set_locale,
            i18n_service::get_text,
            i18n_service::get_supported_locales,
            get_profiles_info,
            get_profiles_list,
            get_profile_status,
            load_profile_content,
            save_profile,
            create_new_profile,
            delete_profile,
            validate_json_content,
            close_settings_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    if cfg!(debug_assertions) {
        log::info!("CCCS application starting in development mode");
    }

    // Hide dock icon on macOS to make this a pure tray application
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let app_handle = app.handle().clone();

    // Initialize basic services with error handling
    match initialize_services(app) {
        Ok(()) => {
            log::info!("Services initialized successfully");
        }
        Err(e) => {
            log::error!("Failed to initialize services: {}", e);
            // Continue anyway - the app can still function without some services
        }
    }

    // Initialize CCCS app and store it in Tauri state
    // This needs to be done synchronously to ensure proper state management
    match initialize_cccs_app(app_handle.clone()) {
        Ok(cccs_app) => {
            log::info!("CCCS app created successfully");
            // Store the app instance
            app.manage(Arc::new(Mutex::new(cccs_app)));

            // Defer actual initialization to avoid blocking startup
            let app_handle_clone = app_handle.clone();
            std::thread::spawn(move || {
                // Add delay to ensure UI is ready
                std::thread::sleep(std::time::Duration::from_millis(2000));

                log::info!("Starting delayed CCCS initialization");

                // Get the app instance and initialize it
                if let Some(app_state) = app_handle_clone.try_state::<Arc<Mutex<App>>>() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut app = app_state.lock().unwrap();
                        match app.initialize().await {
                            Ok(()) => {
                                log::info!("CCCS application initialized successfully");
                            }
                            Err(e) => {
                                log::error!("Failed to initialize CCCS app: {}", e);
                            }
                        }
                    });
                }
            });
        }
        Err(e) => {
            log::error!("Failed to create CCCS app: {}", e);
            // App will continue to run in basic mode
        }
    }

    log::info!("CCCS setup completed");
    Ok(())
}

/// Initialize Tauri state services
fn initialize_services(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize settings service
    match settings_service::SettingsService::new() {
        Ok(settings_service) => {
            app.manage(std::sync::Mutex::new(settings_service));
            log::info!("Settings service initialized");
        }
        Err(e) => {
            log::error!("Failed to initialize settings service: {}", e);
            // Use default settings service
            let default_settings = settings_service::SettingsService::with_defaults();
            app.manage(std::sync::Mutex::new(default_settings));
        }
    }

    // Initialize i18n service
    let i18n_service = i18n_service::I18nService::new();
    app.manage(std::sync::Mutex::new(i18n_service));
    log::info!("I18n service initialized");

    // Note: Config service will be initialized as part of the App instance

    Ok(())
}

/// Create the main CCCS application
fn initialize_cccs_app(app_handle: AppHandle) -> Result<App, Box<dyn std::error::Error>> {
    log::info!("Creating CCCS application instance");

    // Create the main app with error handling
    let cccs_app = match app::App::new(app_handle.clone()) {
        Ok(app) => {
            log::info!("App instance created successfully");
            app
        }
        Err(e) => {
            log::error!("Failed to create App instance: {}", e);
            return Err(e.into());
        }
    };

    Ok(cccs_app)
}
