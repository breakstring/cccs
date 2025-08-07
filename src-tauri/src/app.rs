// Application lifecycle management for CCCS
use crate::{
    claude_detector::ClaudeDetector, config_service::ConfigService, i18n_service::I18nService,
    monitor_service::MonitorService, settings_service::SettingsService, tray_service::TrayService,
    AppError, AppResult,
};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Listener, Manager};

pub struct App {
    config_service: Arc<Mutex<ConfigService>>,
    tray_service: Arc<Mutex<TrayService>>,
    monitor_service: Arc<Mutex<MonitorService>>,
    settings_service: Arc<Mutex<SettingsService>>,
    i18n_service: Arc<Mutex<I18nService>>,
    app_handle: AppHandle,
    is_initialized: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(app_handle: AppHandle) -> AppResult<Self> {
        log::info!("Creating new CCCS application instance");

        let settings_service = Arc::new(Mutex::new(SettingsService::new()?));
        let i18n_service = Arc::new(Mutex::new(I18nService::new()));

        // Get settings for monitor interval
        let monitor_interval = {
            let settings = settings_service.lock().unwrap();
            settings.get_current_settings().monitor_interval_minutes
        };

        // These will be initialized later during the initialization process
        let config_service = Arc::new(Mutex::new(ConfigService::new(std::env::temp_dir()))); // Placeholder
        let tray_service = Arc::new(Mutex::new(TrayService::new(app_handle.clone())));
        let monitor_service = Arc::new(Mutex::new(MonitorService::new(monitor_interval)));

        Ok(Self {
            config_service,
            tray_service,
            monitor_service,
            settings_service,
            i18n_service,
            app_handle,
            is_initialized: false,
        })
    }

    /// Initialize the application
    pub async fn initialize(&mut self) -> AppResult<()> {
        log::info!("Initializing CCCS application");

        if self.is_initialized {
            log::warn!("Application already initialized");
            return Ok(());
        }

        // Step 1: Detect Claude Code installation
        let claude_dir = match ClaudeDetector::detect_claude_installation() {
            Ok(dir) => dir,
            Err(AppError::ClaudeNotFound) => {
                log::info!("Claude Code not found, showing directory picker");

                match ClaudeDetector::show_directory_picker(&self.app_handle).await? {
                    Some(dir) => dir,
                    None => {
                        log::info!("User cancelled directory selection, exiting");
                        self.app_handle.exit(0);
                        return Ok(());
                    }
                }
            }
            Err(e) => return Err(e),
        };

        // Step 2: Validate default configuration
        ClaudeDetector::validate_default_config(&claude_dir)?;

        // Step 3: Initialize configuration service with real Claude directory
        {
            let mut config_service = self.config_service.lock().unwrap();
            *config_service = ConfigService::new(claude_dir.clone());

            // Scan for profiles
            config_service.scan_profiles()?;
        }

        // Step 4: Setup file monitoring
        self.setup_monitoring().await?;

        // Step 5: Create system tray
        self.setup_tray().await?;

        // Step 6: Setup event listeners
        self.setup_event_listeners().await?;

        self.is_initialized = true;
        log::info!("CCCS application initialized successfully");

        Ok(())
    }

    /// Setup file monitoring
    async fn setup_monitoring(&self) -> AppResult<()> {
        log::info!("Setting up file monitoring");

        let config_service = Arc::clone(&self.config_service);
        let tray_service = Arc::clone(&self.tray_service);
        let app_handle = self.app_handle.clone();

        let mut monitor_service = self.monitor_service.lock().unwrap();

        // Add files to monitor
        let monitored_files = {
            let config = config_service.lock().unwrap();
            config.get_monitored_files()
        };

        for file in monitored_files {
            monitor_service.add_file_to_monitor(file);
        }

        // Start monitoring if auto-start is enabled
        let should_auto_start = {
            let settings = self.settings_service.lock().unwrap();
            settings.get_current_settings().auto_start_monitoring
        };

        if should_auto_start {
            let callback = move |changes: Vec<crate::ConfigFileChange>| {
                log::info!("File changes detected: {} files changed", changes.len());

                // Update configuration service
                if let Ok(mut config) = config_service.lock() {
                    if let Err(e) = config.refresh_profile_status() {
                        log::error!("Failed to refresh profile status: {}", e);
                    }

                    // Update tray menu with detailed status
                    if let Ok(mut tray) = tray_service.lock() {
                        let profiles = config.get_profiles();
                        let statuses = config.compare_profiles();
                        if let Err(e) = tray.update_menu_with_detailed_status(profiles, &statuses) {
                            log::error!("Failed to update tray menu: {}", e);
                        }
                    }
                }

                // Emit event to notify frontend
                let _ = app_handle.emit("profiles_changed", ());
            };

            monitor_service.start_monitoring(callback)?;
        }

        Ok(())
    }

    /// Setup system tray
    async fn setup_tray(&self) -> AppResult<()> {
        log::info!("Setting up system tray");

        let mut tray_service = self.tray_service.lock().unwrap();
        tray_service.create_tray()?;

        // Update tray menu with initial profiles and detailed status
        let (profiles, statuses) = {
            let config = self.config_service.lock().unwrap();
            let profiles = config.get_profiles().to_vec();
            let statuses = config.compare_profiles();
            (profiles, statuses)
        };

        tray_service.update_menu_with_detailed_status(&profiles, &statuses)?;

        // Set tooltip
        let tooltip = {
            let i18n = self.i18n_service.lock().unwrap();
            let active_profile = profiles
                .iter()
                .enumerate()
                .find(|(i, _)| matches!(statuses[*i], crate::ProfileStatus::FullMatch))
                .map(|(_, p)| p.name.as_str());
            i18n.get_tray_tooltip(profiles.len(), active_profile)
        };
        tray_service.set_tooltip(&tooltip)?;

        Ok(())
    }

    /// Setup event listeners
    async fn setup_event_listeners(&self) -> AppResult<()> {
        log::info!("Setting up event listeners");

        let config_service = Arc::clone(&self.config_service);
        let tray_service = Arc::clone(&self.tray_service);
        let i18n_service = Arc::clone(&self.i18n_service);

        // Listen for profile switch requests from tray
        let config_service_clone = Arc::clone(&config_service);
        let tray_service_clone = Arc::clone(&tray_service);
        let i18n_service_clone = Arc::clone(&i18n_service);
        let _app_handle_for_switch = self.app_handle.clone();

        self.app_handle
            .listen("profile_switch_requested", move |event| {
                // Parse payload manually since as_str() is unstable
                if let Ok(profile_name) = serde_json::from_str::<String>(event.payload()) {
                    log::info!("Profile switch requested: {}", profile_name);

                    // Show switching status
                    if let Ok(mut tray) = tray_service_clone.lock() {
                        let _ = tray.update_profile_status(&profile_name, "❕");
                    }

                    // Perform switch
                    let result = {
                        let mut config = config_service_clone.lock().unwrap();
                        config.switch_profile(&profile_name)
                    };

                    match result {
                        Ok(()) => {
                            log::info!("Profile switched successfully: {}", profile_name);

                            // Update tray menu with detailed status
                            if let (Ok(config), Ok(mut tray)) =
                                (config_service_clone.lock(), tray_service_clone.lock())
                            {
                                let profiles = config.get_profiles();
                                let statuses = config.compare_profiles();
                                let _ = tray.update_menu_with_detailed_status(profiles, &statuses);

                                // Update tooltip
                                if let Ok(i18n) = i18n_service_clone.lock() {
                                    let active_profile = profiles
                                        .iter()
                                        .enumerate()
                                        .find(|(i, _)| {
                                            matches!(statuses[*i], crate::ProfileStatus::FullMatch)
                                        })
                                        .map(|(_, p)| p.name.as_str());
                                    let tooltip =
                                        i18n.get_tray_tooltip(profiles.len(), active_profile);
                                    let _ = tray.set_tooltip(&tooltip);
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to switch profile {}: {}", profile_name, e);

                            // Reset status on error
                            if let Ok(mut tray) = tray_service_clone.lock() {
                                let _ = tray.update_profile_status(&profile_name, "");
                            }
                        }
                    }
                }
            });

        // Listen for settings menu clicks
        let app_handle_clone = self.app_handle.clone();
        self.app_handle.listen("menu_settings_clicked", move |_| {
            log::info!("Settings menu clicked");

            // Simply emit event to trigger settings window creation (no async needed)
            let _ = app_handle_clone.emit("open_settings_window", ());
        });

        // Listen for open settings window requests
        let app_handle_clone2 = self.app_handle.clone();
        self.app_handle.listen("open_settings_window", move |_| {
            log::info!("Opening settings window");

            // Check if settings window already exists and is visible
            if let Some(window) = app_handle_clone2.get_webview_window("settings") {
                log::info!("Settings window already exists, checking visibility and focusing it");
                // Try to show and focus the window - if it was closed, this will make it visible again
                match window.show() {
                    Ok(_) => {
                        log::info!("Successfully showed existing settings window");
                        let _ = window.set_focus();
                        let _ = window.unminimize(); // In case it was minimized
                        return;
                    }
                    Err(e) => {
                        log::warn!("Failed to show existing settings window: {}, creating new one", e);
                        // Continue to create a new window
                    }
                }
            }

            // Calculate adaptive window size based on screen resolution with improved cross-platform logic
            let (window_width, window_height, min_width, min_height) = {
                // Try to get primary monitor size for adaptive sizing
                if let Ok(monitors) = app_handle_clone2.primary_monitor() {
                    if let Some(monitor) = monitors {
                        let screen_size = monitor.size();
                        log::info!("Primary monitor size: {}x{}", screen_size.width, screen_size.height);
                        
                        // Improved adaptive algorithm:
                        // - Use absolute sizing with reasonable bounds
                        // - Consider different screen densities and platforms
                        let (adaptive_width, adaptive_height) = if screen_size.width >= 1920 {
                            // High resolution displays (1920x1080+, 4K, Retina)
                            if screen_size.height >= 1440 {
                                // 4K or ultrawide displays
                                (1000.0, 900.0)
                            } else {
                                // 1920x1080, 2560x1440
                                (900.0, 800.0) 
                            }
                        } else if screen_size.width >= 1366 {
                            // Medium resolution (1366x768, 1440x900)
                            (800.0, 750.0)
                        } else {
                            // Low resolution or small screens
                            (750.0, 700.0)
                        };
                        
                        // Set conservative minimum sizes to ensure UI elements fit properly
                        let min_w = 500.0;
                        let min_h = 600.0;
                        
                        log::info!("Calculated adaptive window size: {}x{} (min: {}x{}) for screen {}x{}", 
                                 adaptive_width, adaptive_height, min_w, min_h, screen_size.width, screen_size.height);
                        
                        (adaptive_width, adaptive_height, min_w, min_h)
                    } else {
                        log::warn!("No primary monitor found, using default sizes");
                        (900.0, 800.0, 500.0, 600.0) // Conservative defaults
                    }
                } else {
                    log::warn!("Failed to get monitor info, using default sizes");
                    (900.0, 800.0, 500.0, 600.0) // Conservative defaults
                }
            };

            // Create settings window
            match tauri::WebviewWindowBuilder::new(
                &app_handle_clone2,
                "settings",
                tauri::WebviewUrl::App("settings.html".into()),
            )
            .title("CCCS Settings")
            .inner_size(window_width, window_height)
            .min_inner_size(min_width, min_height)
            .center()
            .resizable(true)
            .visible(false) // Start hidden to prevent flashing
            .on_page_load(|window, _payload| {
                // Inject initialization script after page loads
                log::info!("Settings page loaded, injecting init script");
                let init_script = r#"
                    console.log('Page load hook: Checking Tauri API...');
                    if (window.__TAURI__) {
                        console.log('Tauri API is available!');
                    } else {
                        console.error('Tauri API is NOT available in page load hook');
                    }
                "#;
                if let Err(e) = window.eval(init_script) {
                    log::error!("Failed to inject init script: {}", e);
                }
                
                // Show window after a fixed delay to allow for content loading
                let window_clone = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(300)); // Increased delay
                    let _ = window_clone.show();
                    let _ = window_clone.set_focus();
                });
            })
            .build()
            {
                Ok(_window) => {
                    log::info!("Settings window created successfully");
                    // Don't call show() here - it will be called after page load
                }
                Err(e) => {
                    log::error!("Failed to create settings window: {}", e);
                }
            }
        });

        // Listen for tray icon hover events
        let config_service = Arc::clone(&self.config_service);
        let tray_service = Arc::clone(&self.tray_service);
        let _app_handle_for_hover = self.app_handle.clone();
        self.app_handle.listen("tray_icon_hover", move |_| {
            log::info!("Tray icon hover detected, refreshing profiles");

            // Refresh profiles synchronously
            if let Ok(mut config) = config_service.lock() {
                if let Err(e) = config.scan_profiles() {
                    log::error!("Failed to scan profiles on hover: {}", e);
                    return;
                }

                // Update tray menu with fresh profiles and detailed status
                if let Ok(mut tray) = tray_service.lock() {
                    let profiles = config.get_profiles();
                    let statuses = config.compare_profiles();
                    let _ = tray.update_menu_with_detailed_status(profiles, &statuses);
                }
            }
        });

        // Listen for app exit requests
        let monitor_service = Arc::clone(&self.monitor_service);
        self.app_handle.listen("app_exit_requested", move |_| {
            log::info!("Application exit requested");

            // Stop monitoring
            if let Ok(mut monitor) = monitor_service.lock() {
                monitor.stop_monitoring();
            }

            log::info!("Application cleanup completed");
        });

        Ok(())
    }

    /// Update monitor interval
    #[allow(dead_code)]
    pub async fn update_monitor_interval(&self, minutes: u64) -> AppResult<()> {
        log::info!("Updating monitor interval to {} minutes", minutes);

        let mut monitor_service = self.monitor_service.lock().unwrap();
        monitor_service.set_monitor_interval(minutes)?;

        Ok(())
    }

    /// Shutdown the application gracefully
    #[allow(dead_code)]
    pub async fn shutdown(&self) -> AppResult<()> {
        log::info!("Shutting down CCCS application");

        // Stop monitoring
        {
            let mut monitor_service = self.monitor_service.lock().unwrap();
            monitor_service.stop_monitoring();
        }

        // Clean up resources
        log::info!("Application shutdown completed");

        Ok(())
    }

    /// Check if application is initialized
    #[allow(dead_code)]
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Get reference to config service
    pub fn get_config_service(&self) -> Arc<Mutex<ConfigService>> {
        Arc::clone(&self.config_service)
    }

    /// Get reference to settings service for testing
    #[cfg(test)]
    pub fn get_settings_service(&self) -> Arc<Mutex<SettingsService>> {
        Arc::clone(&self.settings_service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_structure() {
        // Simple compilation test for the App structure
        assert!(true);
    }
}
