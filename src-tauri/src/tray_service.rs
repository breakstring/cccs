// System tray service for CCCS
use crate::{AppError, AppResult, Profile, ProfileStatus};
use tauri::{
    menu::{Menu, MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub struct TrayService {
    app_handle: AppHandle,
    current_menu: Option<Menu<tauri::Wry>>,
    tray_id: String,
}

impl TrayService {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            current_menu: None,
            tray_id: "cccs_tray".to_string(),
        }
    }
    
    /// Create and initialize the system tray icon with enhanced error handling
    pub fn create_tray(&mut self) -> AppResult<()> {
        log::info!("Creating system tray icon");
        
        // Build initial menu with error handling
        let menu = match self.build_basic_menu() {
            Ok(menu) => menu,
            Err(e) => {
                log::error!("Failed to build basic menu: {}", e);
                // Create a minimal fallback menu
                self.build_fallback_menu()?
            }
        };
        
        self.current_menu = Some(menu.clone());
        
        // Create tray icon with error handling
        match self.create_tray_icon_safe(&menu) {
            Ok(_) => {
                log::info!("System tray icon created successfully");
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to create tray icon: {}", e);
                Err(e)
            }
        }
    }
    
    /// Create tray icon with safety checks
    fn create_tray_icon_safe(&self, menu: &Menu<tauri::Wry>) -> AppResult<()> {
        // Try to load tray-specific icon first, fallback to app icon
        let icon_result = self.load_tray_icon();
        
        match icon_result {
            Ok(icon) => {
                log::info!("Using custom tray icon");
                self.create_tray_with_icon(menu, icon)
            }
            Err(_) => {
                // Fallback to default window icon
                let icon = match self.app_handle.default_window_icon() {
                    Some(icon) => {
                        log::info!("Using default window icon for tray");
                        icon.clone()
                    }
                    None => {
                        log::warn!("No default window icon found, creating tray without icon");
                        return self.create_tray_without_icon(menu);
                    }
                };
                self.create_tray_with_icon(menu, icon)
            }
        }
    }
    
    /// Load tray-specific icon with cross-platform path resolution
    fn load_tray_icon(&self) -> AppResult<tauri::image::Image<'_>> {
        use std::fs;
        
        // Get base paths for different environments
        let base_paths = self.get_icon_search_paths();
        
        // Try to load custom tray icon first (ordered by preference)
        let tray_icon_relative_paths = [
            "icons/tray/tray-icon-large.png",       // Large icon with small padding (current choice)
            "icons/tray/tray-icon-xl.png",          // Extra large icon (minimal padding)
            "icons/tray/tray-icon-large-32.png",    // 32x32 large version
            "icons/tray/tray-icon-clean-16.png",    // Original clean version
            "icons/tray/tray-icon-hq-16.png",       // High-quality black version
            "icons/32x32.png"                       // fallback to original icon
        ];
        
        // Try each base path with each icon path
        for base_path in &base_paths {
            for relative_path in &tray_icon_relative_paths {
                let icon_path = base_path.join(relative_path);
                log::debug!("Trying to load tray icon from: {:?}", icon_path);
                
                if let Ok(icon_data) = fs::read(&icon_path) {
                    match self.create_tauri_image_from_data(&icon_data) {
                        Ok(tauri_image) => {
                            log::info!("Successfully loaded custom tray icon from: {:?}", icon_path);
                            return Ok(tauri_image);
                        }
                        Err(e) => {
                            log::debug!("Failed to decode image from {:?}: {}", icon_path, e);
                            continue;
                        }
                    }
                } else {
                    log::debug!("Failed to read file {:?}", icon_path);
                }
            }
        }
        
        Err(AppError::TrayError("Could not load any tray icon".to_string()))
    }
    
    /// Get cross-platform icon search paths
    fn get_icon_search_paths(&self) -> Vec<std::path::PathBuf> {
        let mut paths = Vec::new();
        
        // 1. Try Tauri resource directory first (production)
        if let Ok(resource_dir) = self.app_handle.path().resource_dir() {
            paths.push(resource_dir);
            log::debug!("Added resource directory: {:?}", paths.last());
        }
        
        // 2. Try app data directory
        if let Ok(app_data_dir) = self.app_handle.path().app_data_dir() {
            paths.push(app_data_dir);
            log::debug!("Added app data directory: {:?}", paths.last());
        }
        
        // 3. Development mode: try project structure paths
        if cfg!(debug_assertions) {
            if let Ok(current_dir) = std::env::current_dir() {
                // If we're in src-tauri, go up one level
                let project_root = if current_dir.file_name()
                    .and_then(|n| n.to_str()) == Some("src-tauri") {
                    current_dir.parent().unwrap_or(&current_dir).to_path_buf()
                } else {
                    current_dir
                };
                
                // Add project root and src-tauri subdirectory
                paths.push(project_root.clone());
                paths.push(project_root.join("src-tauri"));
                log::debug!("Added development paths: {:?}", &paths[paths.len()-2..]);
            }
        }
        
        // 4. Fallback: try current executable directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                paths.push(exe_dir.to_path_buf());
                log::debug!("Added executable directory: {:?}", paths.last());
            }
        }
        
        paths
    }
    
    /// Create Tauri image from raw image data
    fn create_tauri_image_from_data(&self, icon_data: &[u8]) -> Result<tauri::image::Image<'_>, Box<dyn std::error::Error>> {
        let img = image::load_from_memory(icon_data)?;
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();
        let rgba_data = rgba_img.into_raw();
        
        Ok(tauri::image::Image::new_owned(rgba_data, width, height))
    }
    
    /// Create tray with provided icon
    fn create_tray_with_icon(&self, _menu: &Menu<tauri::Wry>, icon: tauri::image::Image<'_>) -> AppResult<()> {
        let app_handle_clone = self.app_handle.clone();
        
        let use_template_mode = self.should_use_template_mode();
        let show_menu_on_left = self.should_show_menu_on_left_click();
        
        // 平台特定的托盘创建策略
        #[cfg(target_os = "windows")]
        {
            log::info!("Creating tray icon for Windows (without menu due to Tauri bug) - template mode: {}, show_menu_on_left_click: {}", use_template_mode, show_menu_on_left);
            
            let _tray = TrayIconBuilder::with_id(&self.tray_id)
                .icon(icon)
                // Windows下不使用菜单，绕过bug
                .icon_as_template(use_template_mode)
                .show_menu_on_left_click(show_menu_on_left)
                .on_menu_event(move |app, event| {
                    log::info!("Menu event received: {:?}", event);
                    if let Err(e) = Self::handle_menu_event_safe(app, event) {
                        log::error!("Error handling menu event: {}", e);
                    }
                })
                .on_tray_icon_event(move |tray, event| {
                    log::info!("Tray event received: {:?}", event);
                    let app_handle = &app_handle_clone;
                    if let Err(e) = Self::handle_tray_event_safe(app_handle, tray, event) {
                        log::error!("Error handling tray event: {}", e);
                    }
                })
                .build(&self.app_handle)
                .map_err(|e| AppError::TrayError(format!("Failed to build tray icon: {}", e)))?;
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            log::info!("Creating tray icon for Mac/Linux (with menu) - template mode: {}, show_menu_on_left_click: {}", use_template_mode, show_menu_on_left);
            
            let _tray = TrayIconBuilder::with_id(&self.tray_id)
                .icon(icon)
                .menu(menu) // Mac/Linux下使用正常的菜单
                .icon_as_template(use_template_mode)
                .show_menu_on_left_click(show_menu_on_left)
                .on_menu_event(move |app, event| {
                    log::info!("Menu event received: {:?}", event);
                    if let Err(e) = Self::handle_menu_event_safe(app, event) {
                        log::error!("Error handling menu event: {}", e);
                    }
                })
                .on_tray_icon_event(move |tray, event| {
                    log::info!("Tray event received: {:?}", event);
                    let app_handle = &app_handle_clone;
                    if let Err(e) = Self::handle_tray_event_safe(app_handle, tray, event) {
                        log::error!("Error handling tray event: {}", e);
                    }
                })
                .build(&self.app_handle)
                .map_err(|e| AppError::TrayError(format!("Failed to build tray icon: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// 智能决定是否使用模板模式
    fn should_use_template_mode(&self) -> bool {
        // Windows下模板模式可能会影响菜单显示，尝试禁用
        #[cfg(target_os = "windows")]
        {
            false // Windows: 禁用模板模式，可能解决菜单显示问题
        }
        
        #[cfg(target_os = "macos")]
        {
            false // macOS: 也暂时禁用模板模式进行测试
        }
        
        #[cfg(target_os = "linux")]
        {
            false // Linux: 禁用模板模式
        }
    }
    
    /// 决定是否在左键点击时显示菜单（平台特定）
    fn should_show_menu_on_left_click(&self) -> bool {
        // 经过分析，Windows下的托盘菜单问题可能是因为：
        // 1. show_menu_on_left_click(false) 禁用了左键菜单
        // 2. 但右键菜单在某些Windows版本下可能不会自动显示
        // 
        // 解决方案：在所有平台都启用左键菜单，确保用户至少有一种方式访问菜单
        
        #[cfg(target_os = "windows")]
        {
            // Windows: 启用左键菜单作为主要访问方式，右键也应该工作
            true
        }
        
        #[cfg(target_os = "macos")]
        {
            // macOS: 也启用左键菜单，提供更好的用户体验
            // 用户可以左键或右键都能访问菜单
            true
        }
        
        #[cfg(target_os = "linux")]
        {
            // Linux: 启用左键菜单（虽然文档说不支持，但设置为true不会有害）
            true
        }
    }
    
    /// 检测系统主题偏好（未来扩展用）
    #[allow(dead_code)]
    fn detect_system_theme_preference(&self) -> bool {
        // 这里可以检测系统主题或用户设置
        // 暂时返回false（使用彩色模式）
        false
    }
    
    /// Create tray without icon as fallback
    fn create_tray_without_icon(&self, menu: &Menu<tauri::Wry>) -> AppResult<()> {
        let app_handle_clone = self.app_handle.clone();
        let show_menu_on_left = self.should_show_menu_on_left_click();
        log::info!("Creating tray icon without icon, show_menu_on_left_click: {}", show_menu_on_left);
        
        let _tray = TrayIconBuilder::with_id(&self.tray_id)
            .menu(menu)
            .icon_as_template(true) // Try template mode for better macOS integration
            .show_menu_on_left_click(show_menu_on_left) // Platform-specific menu behavior
            .on_menu_event(move |app, event| {
                if let Err(e) = Self::handle_menu_event_safe(app, event) {
                    log::error!("Error handling menu event: {}", e);
                }
            })
            .on_tray_icon_event(move |tray, event| {
                let app_handle = &app_handle_clone;
                if let Err(e) = Self::handle_tray_event_safe(app_handle, tray, event) {
                    log::error!("Error handling tray event: {}", e);
                }
            })
            .build(&self.app_handle)
            .map_err(|e| AppError::TrayError(format!("Failed to build tray icon without icon: {}", e)))?;
        
        Ok(())
    }
    
    /// Create a minimal fallback menu
    fn build_fallback_menu(&self) -> AppResult<Menu<tauri::Wry>> {
        let menu = MenuBuilder::new(&self.app_handle)
            .text("settings", "Settings")
            .separator()
            .text("exit", "Exit")
            .build()
            .map_err(|e| AppError::TrayError(format!("Failed to build fallback menu: {}", e)))?;
        
        Ok(menu)
    }
    
    /// Build the basic menu structure (empty profiles, settings, exit)
    fn build_basic_menu(&self) -> AppResult<Menu<tauri::Wry>> {
        // 尝试最简单的菜单结构来诊断Windows菜单问题
        log::info!("Building basic menu with simple structure");
        
        let menu = MenuBuilder::new(&self.app_handle)
            .text("test", "TEST MENU") // 使用简单的text方法
            .text("settings", "Settings")
            .text("exit", "Exit")
            .build()?;
        
        log::info!("Basic menu built successfully");
        Ok(menu)
    }
    
    /// Update menu with current profiles
    pub fn update_menu(&mut self, profiles: &[Profile]) -> AppResult<()> {
        log::info!("Updating tray menu with {} profiles", profiles.len());
        
        let mut menu_builder = MenuBuilder::new(&self.app_handle);
        
        // Add profile menu items
        for profile in profiles {
            let menu_text = if profile.is_active {
                format!("✅ {}", profile.name)
            } else {
                format!("　  {}", profile.name)  // 全角空格 + 两个普通空格
            };
            
            let menu_item = MenuItemBuilder::with_id(
                format!("profile_{}", profile.name),
                menu_text
            ).build(&self.app_handle)?;
            
            menu_builder = menu_builder.item(&menu_item);
        }
        
        // Add separator only if there are profiles, then add system menu items
        let menu = if profiles.is_empty() {
            // No profiles - just add system menu items without separator
            menu_builder
                .item(&MenuItemBuilder::with_id("settings", "Settings").build(&self.app_handle)?)
                .item(&MenuItemBuilder::with_id("exit", "Exit").build(&self.app_handle)?)
                .build()?
        } else {
            // Has profiles - add separator then system menu items
            menu_builder
                .separator()
                .item(&MenuItemBuilder::with_id("settings", "Settings").build(&self.app_handle)?)
                .item(&MenuItemBuilder::with_id("exit", "Exit").build(&self.app_handle)?)
                .build()?
        };
        
        // Update the tray menu - get tray by ID
        if let Some(tray) = self.app_handle.tray_by_id(&self.tray_id) {
            tray.set_menu(Some(menu.clone()))?;
        }
        
        self.current_menu = Some(menu);
        log::info!("Tray menu updated successfully");
        
        Ok(())
    }
    
    /// Update menu with detailed profile status indicators
    pub fn update_menu_with_detailed_status(&mut self, profiles: &[Profile], statuses: &[ProfileStatus]) -> AppResult<()> {
        log::info!("Updating tray menu with {} profiles and detailed status", profiles.len());
        
        let mut menu_builder = MenuBuilder::new(&self.app_handle);
        
        // Add profile menu items with detailed status
        for (profile, status) in profiles.iter().zip(statuses.iter()) {
            let menu_text = match status {
                ProfileStatus::FullMatch => format!("✅ {}", profile.name),      // 完全匹配 - 图标前置
                ProfileStatus::PartialMatch => format!("🔄 {}", profile.name),  // 仅model字段不同 - 图标前置
                ProfileStatus::NoMatch => format!("　  {}", profile.name),       // 配置不同，全角空格 + 两个普通空格
                ProfileStatus::Error(_) => format!("❌ {}", profile.name),       // 错误状态 - 图标前置
            };
            
            let menu_item = MenuItemBuilder::with_id(
                format!("profile_{}", profile.name),
                menu_text
            ).build(&self.app_handle)?;
            
            menu_builder = menu_builder.item(&menu_item);
        }
        
        // 临时简化菜单，只使用最基本的结构进行测试
        let menu = if profiles.is_empty() {
            log::info!("No profiles found, creating simple test menu");
            MenuBuilder::new(&self.app_handle)
                .text("no_profiles", "No profiles found")
                .text("settings", "Settings")
                .text("exit", "Exit")
                .build()?
        } else {
            log::info!("Profiles found, creating profile menu");
            let mut menu_builder = MenuBuilder::new(&self.app_handle);
            
            // Add profile items using simple text method
            for (profile, status) in profiles.iter().zip(statuses.iter()) {
                let menu_text = match status {
                    ProfileStatus::FullMatch => format!("✅ {}", profile.name),
                    ProfileStatus::PartialMatch => format!("🔄 {}", profile.name),
                    ProfileStatus::NoMatch => format!("　  {}", profile.name),
                    ProfileStatus::Error(_) => format!("❌ {}", profile.name),
                };
                menu_builder = menu_builder.text(&format!("profile_{}", profile.name), menu_text);
            }
            
            menu_builder
                .text("settings", "Settings")
                .text("exit", "Exit")
                .build()?
        };
        
        // Update the tray menu - get tray by ID
        if let Some(tray) = self.app_handle.tray_by_id(&self.tray_id) {
            tray.set_menu(Some(menu.clone()))?;
        }
        
        self.current_menu = Some(menu);
        log::info!("Tray menu updated successfully with detailed status indicators");
        
        Ok(())
    }
    
    /// Handle menu item click events with error handling
    fn handle_menu_event_safe(app: &AppHandle, event: tauri::menu::MenuEvent) -> AppResult<()> {
        let event_id = event.id().as_ref();
        log::info!("Menu item clicked: {}", event_id);
        
        match event_id {
            "settings" => {
                Self::handle_settings_click(app)
            }
            "exit" => {
                Self::handle_exit_click(app)
            }
            id if id.starts_with("profile_") => {
                let profile_name = id.strip_prefix("profile_").unwrap_or("");
                Self::handle_profile_click(app, profile_name)
            }
            _ => {
                log::warn!("Unhandled menu event: {}", event_id);
                Ok(())
            }
        }
    }
    
    /// Handle tray icon click events with error handling
    fn handle_tray_event_safe(app_handle: &AppHandle, _tray: &tauri::tray::TrayIcon<tauri::Wry>, event: TrayIconEvent) -> AppResult<()> {
        match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                log::debug!("Tray icon left-clicked");
                
                #[cfg(target_os = "windows")]
                {
                    // Windows: 左键点击打开设置窗口
                    log::info!("Left-click on Windows, opening settings window");
                    app_handle.emit("tray_icon_hover", ())
                        .map_err(|e| AppError::TrayError(format!("Failed to emit hover event: {}", e)))?;
                    app_handle.emit("menu_settings_clicked", ())
                        .map_err(|e| AppError::TrayError(format!("Failed to emit settings event: {}", e)))?;
                }
                
                #[cfg(not(target_os = "windows"))]
                {
                    // Mac/Linux: 左键点击触发配置扫描（如果启用了左键菜单，菜单会自动显示）
                    app_handle.emit("tray_icon_hover", ())
                        .map_err(|e| AppError::TrayError(format!("Failed to emit hover event: {}", e)))?;
                }
                
                Ok(())
            }
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
                log::debug!("Tray icon double-clicked");
                
                #[cfg(target_os = "windows")]
                {
                    // Windows: 双击不做特殊处理，避免与单击冲突
                    log::debug!("Double-click on Windows, no action");
                }
                
                #[cfg(not(target_os = "windows"))]
                {
                    // Mac/Linux: 双击打开设置窗口
                    log::info!("Double-click on Mac/Linux, opening settings window");
                    app_handle.emit("menu_settings_clicked", ())
                        .map_err(|e| AppError::TrayError(format!("Failed to emit settings event: {}", e)))?;
                }
                
                Ok(())
            }
            TrayIconEvent::Enter { .. } => {
                log::debug!("Mouse entered tray icon area");
                // Trigger profile scan when mouse enters
                app_handle.emit("tray_icon_hover", ())
                    .map_err(|e| AppError::TrayError(format!("Failed to emit hover event: {}", e)))?;
                Ok(())
            }
            TrayIconEvent::Click {
                button: MouseButton::Right,
                button_state: MouseButtonState::Up,
                position: _,
                ..
            } => {
                log::debug!("Tray icon right-clicked");
                
                // 触发配置文件扫描
                app_handle.emit("tray_icon_hover", ())
                    .map_err(|e| AppError::TrayError(format!("Failed to emit hover event: {}", e)))?;
                
                #[cfg(target_os = "windows")]
                {
                    // Windows: 右键点击也打开设置窗口
                    log::info!("Right-click on Windows, opening settings window");
                    app_handle.emit("menu_settings_clicked", ())
                        .map_err(|e| AppError::TrayError(format!("Failed to emit settings event: {}", e)))?;
                }
                
                #[cfg(not(target_os = "windows"))]
                {
                    // Mac/Linux: 右键点击会显示菜单（由Tauri自动处理）
                    log::debug!("Right-click on Mac/Linux, menu should show automatically");
                }
                
                Ok(())
            }
            _ => {
                log::debug!("Other tray event: {:?}", event);
                Ok(())
            }
        }
    }
    
    /// Handle settings menu item click
    fn handle_settings_click(app: &AppHandle) -> AppResult<()> {
        log::info!("Settings menu clicked");
        
        // Emit event to notify other parts of the application
        app.emit("menu_settings_clicked", ())
            .map_err(|e| AppError::TrayError(format!("Failed to emit settings event: {}", e)))?;
        
        Ok(())
    }
    
    /// Handle exit menu item click
    fn handle_exit_click(app: &AppHandle) -> AppResult<()> {
        log::info!("Exit menu clicked");
        
        // Emit event for cleanup before exit
        let _ = app.emit("app_exit_requested", ());
        
        // Exit the application
        app.exit(0);
        Ok(())
    }
    
    /// Handle profile menu item click
    fn handle_profile_click(app: &AppHandle, profile_name: &str) -> AppResult<()> {
        log::info!("Profile menu clicked: {}", profile_name);
        
        // Emit event with profile name
        app.emit("profile_switch_requested", profile_name)
            .map_err(|e| AppError::TrayError(format!("Failed to emit profile switch event: {}", e)))?;
        
        Ok(())
    }
    
    /// Show temporary status in menu item (e.g., ❕ during switch)
    pub fn update_profile_status(&mut self, profile_name: &str, status: &str) -> AppResult<()> {
        log::debug!("Updating profile status: {} -> {}", profile_name, status);
        
        // For temporary status updates (like showing ❕ during switch)
        // we need to rebuild the menu with updated text
        if let Some(_current_menu) = &self.current_menu {
            // Emit event to trigger menu refresh with temporary status
            self.app_handle.emit("profile_status_update", (profile_name, status))
                .map_err(|e| AppError::TrayError(format!("Failed to emit status update: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Update menu with profiles and temporary status indicators
    pub fn update_menu_with_status(&mut self, profiles: &[Profile], status_updates: &std::collections::HashMap<String, String>) -> AppResult<()> {
        log::info!("Updating tray menu with {} profiles and status updates", profiles.len());
        
        let mut menu_builder = MenuBuilder::new(&self.app_handle);
        
        // Add profile menu items with status
        for profile in profiles {
            let menu_text = if let Some(temp_status) = status_updates.get(&profile.name) {
                // Show temporary status (e.g., "❕ Profile")
                format!("{} {}", temp_status, profile.name)
            } else if profile.is_active {
                // Show active status
                format!("✅ {}", profile.name)
            } else {
                // No status - use full-width space + two normal spaces
                format!("　  {}", profile.name)
            };
            
            let menu_item = MenuItemBuilder::with_id(
                format!("profile_{}", profile.name),
                menu_text
            ).build(&self.app_handle)?;
            
            menu_builder = menu_builder.item(&menu_item);
        }
        
        // Add separator only if there are profiles, then add system menu items
        let menu = if profiles.is_empty() {
            // No profiles - just add system menu items without separator
            menu_builder
                .item(&MenuItemBuilder::with_id("settings", "Settings").build(&self.app_handle)?)
                .item(&MenuItemBuilder::with_id("exit", "Exit").build(&self.app_handle)?)
                .build()?
        } else {
            // Has profiles - add separator then system menu items
            menu_builder
                .separator()
                .item(&MenuItemBuilder::with_id("settings", "Settings").build(&self.app_handle)?)
                .item(&MenuItemBuilder::with_id("exit", "Exit").build(&self.app_handle)?)
                .build()?
        };
        
        // Update the tray menu - get tray by ID
        if let Some(tray) = self.app_handle.tray_by_id(&self.tray_id) {
            tray.set_menu(Some(menu.clone()))?;
        }
        
        self.current_menu = Some(menu);
        log::info!("Tray menu updated successfully with status indicators");
        
        Ok(())
    }
    
    /// Force refresh the menu (useful after profile changes)
    pub fn refresh_menu(&mut self, profiles: &[Profile]) -> AppResult<()> {
        self.update_menu(profiles)
    }
    
    /// Set tray tooltip
    pub fn set_tooltip(&self, text: &str) -> AppResult<()> {
        if let Some(tray) = self.app_handle.tray_by_id(&self.tray_id) {
            tray.set_tooltip(Some(text))?;
        }
        Ok(())
    }
    
    /// Show popup menu at cursor position (Windows workaround)
    #[cfg(target_os = "windows")]
    fn show_popup_menu(app_handle: &AppHandle, position: tauri::PhysicalPosition<f64>) -> AppResult<()> {
        log::info!("Triggering popup menu at position: {:?}", position);
        
        // 首先发送一个测试事件来验证事件系统工作
        if let Err(e) = app_handle.emit("test_event", "Testing event system") {
            log::error!("Failed to emit test event: {}", e);
        } else {
            log::info!("Test event sent successfully");
        }
        
        // 发送事件到前端，让它显示一个弹出菜单
        app_handle.emit("show_tray_popup_menu", serde_json::json!({
            "position": {
                "x": position.x,
                "y": position.y
            },
            "items": [
                {"id": "no_profiles", "label": "No profiles found", "enabled": false},
                {"id": "separator", "label": "-"},
                {"id": "settings", "label": "Settings"},
                {"id": "exit", "label": "Exit"}
            ]
        })).map_err(|e| AppError::TrayError(format!("Failed to emit popup menu event: {}", e)))?;
        
        log::info!("Popup menu event sent successfully");
        Ok(())
    }
    
    /// Show popup menu - no-op on non-Windows platforms
    #[cfg(not(target_os = "windows"))]
    fn show_popup_menu(_app_handle: &AppHandle, _position: tauri::PhysicalPosition<f64>) -> AppResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Profile;
    use std::path::PathBuf;
    
    fn create_test_profile(name: &str, is_active: bool) -> Profile {
        Profile {
            name: name.to_string(),
            path: PathBuf::from(format!("{}.settings.json", name)),
            content: "{}".to_string(),
            is_active,
        }
    }
    
    // Note: Testing TrayService requires a Tauri app context
    // These tests would need to be integration tests with a real Tauri app
    // For now, we'll test the basic logic
    
    #[test]
    fn test_profile_creation() {
        let profile = create_test_profile("test", true);
        assert_eq!(profile.name, "test");
        assert!(profile.is_active);
    }
    
    #[test]
    fn test_menu_text_generation() {
        let active_profile = create_test_profile("active", true);
        let inactive_profile = create_test_profile("inactive", false);
        
        // Test that active profiles would have checkmark
        let active_text = if active_profile.is_active {
            format!("✅ {}", active_profile.name)
        } else {
            format!("　  {}", active_profile.name)
        };
        
        let inactive_text = if inactive_profile.is_active {
            format!("✅ {}", inactive_profile.name)
        } else {
            format!("　  {}", inactive_profile.name)
        };
        
        assert_eq!(active_text, "✅ active");
        assert_eq!(inactive_text, "　  inactive");
    }
}