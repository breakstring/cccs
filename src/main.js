// CCCS Enhanced Settings Page - Main Entry Point
import './style.css'

// Global state
let globalState = {
    currentProfile: null,
    profiles: [],
    isLoading: false,
    hasUnsavedChanges: false,
    lastError: null
};

// Tauri APIs - will be initialized when available
let invoke = null;
let getCurrentWindow = null;

// Internationalization (enhanced with new keys)
const translations = {
    en: {
        // Existing translations
        app_name: 'CCCS',
        settings_title: 'CCCS - Settings',
        app_info_title: 'Application Information',
        app_description: 'CCCS (Claude Code Configuration Switcher) is a tool for quickly switching Claude Code configuration files.',
        status_icons_title: 'Profile Status Icons',
        full_match_description: 'Complete match - configuration fully matches current settings',
        partial_match_description: 'Partial match - identical except model field (auto-updated by Claude Code)',
        error_status_description: 'Error - failed to read or parse configuration file',
        no_match_description: 'No icon - configuration differs from current settings',
        language_settings_title: 'Language Settings',
        interface_language_label: 'Interface language:',
        follow_system: 'Follow system',
        chinese: '中文',
        english: 'English',
        current_status_title: 'Current Status',
        claude_directory_label: 'Claude Code directory:',
        profiles_found_label: 'Configuration files found:',
        close_button: 'Close',
        saving_settings: 'Saving settings...',
        
        // New translations for enhanced UI
        profiles_section_title: 'Profiles',
        about_nav_item: 'About',
        current_profile_title: 'Current',
        json_editor_title: 'Configuration Editor',
        json_editor_hint: 'Edit your configuration in JSON format',
        json_editor_placeholder: 'Loading configuration...',
        save_button: 'Save',
        save_as_button: 'Save As...',
        save_as_modal_title: 'Save As New Profile',
        profile_name_label: 'Profile Name:',
        profile_name_placeholder: 'Enter profile name',
        profile_name_hint: 'Profile names cannot contain special characters: / \\ : * ? " < > |',
        preview_filename_label: 'Preview:',
        cancel_button: 'Cancel',
        
        // Status messages
        loading_profiles: 'Loading profiles...',
        loading_content: 'Loading content...',
        saving_profile: 'Saving profile...',
        save_success: 'Profile saved successfully',
        save_error: 'Failed to save profile',
        validation_error: 'Invalid JSON format',
        profile_created: 'Profile created successfully',
        profile_create_error: 'Failed to create profile',
    },
    zh: {
        // Existing translations
        app_name: 'CCCS',
        settings_title: 'CCCS - 设置',
        app_info_title: '应用程序信息',
        app_description: 'CCCS (Claude Code Configuration Switcher) 是一个用于快速切换 Claude Code 配置文件的工具。',
        status_icons_title: '配置状态图标',
        full_match_description: '完全匹配 - 配置与当前设置完全一致',
        partial_match_description: '部分匹配 - 除model字段外完全一致（Claude Code自动更新）',
        error_status_description: '错误 - 读取或解析配置文件失败',
        no_match_description: '无图标 - 配置与当前设置不同',
        language_settings_title: '语言设置',
        interface_language_label: '界面语言:',
        follow_system: '跟随系统',
        chinese: '中文',
        english: 'English',
        current_status_title: '当前状态',
        claude_directory_label: 'Claude Code 目录:',
        profiles_found_label: '发现的配置文件:',
        close_button: '关闭',
        saving_settings: '正在保存设置...',
        
        // New translations for enhanced UI
        profiles_section_title: '配置文件',
        about_nav_item: '关于',
        current_profile_title: '当前配置',
        json_editor_title: '配置编辑器',
        json_editor_hint: '以JSON格式编辑您的配置',
        json_editor_placeholder: '正在加载配置...',
        save_button: '保存',
        save_as_button: '另存为...',
        save_as_modal_title: '另存为新的配置文件',
        profile_name_label: '配置文件名:',
        profile_name_placeholder: '输入配置文件名',
        profile_name_hint: '配置文件名不能包含特殊字符: / \\ : * ? " < > |',
        preview_filename_label: '预览:',
        cancel_button: '取消',
        
        // Status messages
        loading_profiles: '正在加载配置文件...',
        loading_content: '正在加载内容...',
        saving_profile: '正在保存配置文件...',
        save_success: '配置文件保存成功',
        save_error: '保存配置文件失败',
        validation_error: 'JSON格式无效',
        profile_created: '配置文件创建成功',
        profile_create_error: '创建配置文件失败',
    }
};

let currentLanguage = 'en';

// Utility functions
function detectSystemLanguage() {
    const lang = navigator.language || navigator.languages[0];
    if (lang.startsWith('zh')) {
        return 'zh';
    }
    return 'en';
}

function updateTexts() {
    const t = translations[currentLanguage];
    
    document.querySelectorAll('[data-i18n]').forEach(element => {
        const key = element.getAttribute('data-i18n');
        if (t[key]) {
            element.textContent = t[key];
        }
    });
    
    // Update placeholders
    document.querySelectorAll('[data-i18n-placeholder]').forEach(element => {
        const key = element.getAttribute('data-i18n-placeholder');
        if (t[key]) {
            element.placeholder = t[key];
        }
    });
    
    // Update page title
    document.title = t.settings_title;
}

function showToast(message, type = 'info') {
    const toastContainer = document.getElementById('toast-container');
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    
    toastContainer.appendChild(toast);
    
    // Auto remove after 3 seconds
    setTimeout(() => {
        toast.remove();
    }, 3000);
}

function showLoading(show, message = null) {
    const overlay = document.getElementById('loading-overlay');
    const text = overlay.querySelector('.loading-text');
    
    if (show) {
        if (message) {
            text.textContent = message;
        }
        overlay.style.display = 'flex';
    } else {
        overlay.style.display = 'none';
    }
    
    globalState.isLoading = show;
}

// Navigation Panel Component
class NavigationPanel {
    constructor() {
        this.selectedItem = null;
        this.profileNavList = document.getElementById('profile-nav-list');
        this.aboutNavItem = document.querySelector('.about-nav-item');
        
        this.initializeEventListeners();
    }
    
    initializeEventListeners() {
        // About navigation click
        this.aboutNavItem.addEventListener('click', () => {
            this.handleItemClick('about');
        });
    }
    
    async loadProfiles() {
        try {
            showLoading(true, translations[currentLanguage].loading_profiles);
            
            const profiles = await invoke('get_profiles_list');
            globalState.profiles = profiles;
            
            this.renderProfileList(profiles);
            
            // Select first profile (Current) by default
            if (profiles.length > 0) {
                this.handleItemClick(profiles[0].id);
            }
            
            // Update profiles count in About section
            updateProfilesCount();
            
        } catch (error) {
            console.error('Failed to load profiles:', error);
            showToast(translations[currentLanguage].save_error, 'error');
        } finally {
            showLoading(false);
        }
    }
    
    renderProfileList(profiles) {
        this.profileNavList.innerHTML = '';
        
        profiles.forEach(profile => {
            const listItem = document.createElement('li');
            listItem.className = 'nav-item profile-nav-item';
            listItem.setAttribute('data-profile-id', profile.id);
            
            // Add special class for Current profile
            if (profile.id === 'current') {
                listItem.classList.add('current-profile');
            }
            
            // Create icon span for status
            const iconSpan = document.createElement('span');
            iconSpan.className = 'nav-item-icon';
            
            // Create text span
            const text = document.createElement('span');
            text.className = 'nav-item-text';
            text.textContent = profile.display_name;
            
            listItem.appendChild(iconSpan);
            listItem.appendChild(text);
            
            listItem.addEventListener('click', () => {
                this.handleItemClick(profile.id);
            });
            
            this.profileNavList.appendChild(listItem);
        });
        
        // Load status icons after rendering
        this.loadProfileStatuses();
    }
    
    handleItemClick(profileId) {
        // 只有在真正有内容变更且不是刚创建新profile时才提示
        if (globalState.hasUnsavedChanges && globalState.currentProfile && globalState.currentProfile !== profileId) {
            const t = translations[currentLanguage];
            if (!confirm(t.loading_profiles || 'You have unsaved changes. Do you want to continue?')) {
                return;
            }
        }
        
        this.updateActiveState(profileId);
        
        // Notify content editor
        if (window.contentEditor) {
            window.contentEditor.loadProfile(profileId);
        }
        
        globalState.currentProfile = profileId;
        globalState.hasUnsavedChanges = false;
    }
    
    updateActiveState(profileId) {
        // Remove active class from all items
        document.querySelectorAll('.nav-item').forEach(item => {
            item.classList.remove('active');
        });
        
        // Add active class to selected item
        if (profileId === 'about') {
            // Handle about navigation item
            const aboutItem = document.querySelector('.about-nav-item');
            if (aboutItem) {
                aboutItem.classList.add('active');
                this.selectedItem = profileId;
            }
        } else {
            // Handle profile navigation items
            const selectedItem = document.querySelector(`[data-profile-id="${profileId}"]`);
            if (selectedItem) {
                selectedItem.classList.add('active');
                this.selectedItem = profileId;
            }
        }
    }
    
    refreshProfileList() {
        this.loadProfiles();
    }
    
    // Load profile status icons
    async loadProfileStatuses() {
        try {
            const profiles = globalState.profiles;
            
            for (const profile of profiles) {
                if (profile.id === 'current') {
                    continue; // Skip current profile
                }
                
                // Get profile status using the new command
                const statusIcon = await invoke('get_profile_status', { profileId: profile.id });
                
                // Update the icon in the UI
                const listItem = document.querySelector(`[data-profile-id="${profile.id}"]`);
                if (listItem) {
                    const iconSpan = listItem.querySelector('.nav-item-icon');
                    if (iconSpan) {
                        iconSpan.textContent = statusIcon;
                    }
                }
            }
        } catch (error) {
            console.error('Failed to load profile statuses:', error);
        }
    }
}

// Content Editor Component
class ContentEditor {
    constructor() {
        this.profileTitle = document.getElementById('profile-title');
        this.jsonEditor = document.getElementById('json-editor');
        this.jsonEditorView = document.getElementById('json-editor-view');
        this.aboutView = document.getElementById('about-view');
        this.saveButton = document.getElementById('save-button');
        this.saveAsButton = document.getElementById('save-as-button');
        
        this.currentContent = '';
        this.originalContent = '';
        
        this.initializeEventListeners();
    }
    
    initializeEventListeners() {
        // Content change tracking
        this.jsonEditor.addEventListener('input', () => {
            this.handleContentChange();
        });
        
        // Save button
        this.saveButton.addEventListener('click', () => {
            this.save();
        });
        
        // Save As button
        this.saveAsButton.addEventListener('click', () => {
            this.showSaveAsModal();
        });
    }
    
    async loadProfile(profileId) {
        try {
            showLoading(true, translations[currentLanguage].loading_content);
            
            if (profileId === 'about') {
                this.showAboutContent();
                return;
            }
            
            this.showEditorContent();
            
            // Update title
            const profile = globalState.profiles.find(p => p.id === profileId);
            if (profile) {
                this.profileTitle.textContent = profile.display_name;
            }
            
            // Load content
            const content = await invoke('load_profile_content', { profileId });
            this.jsonEditor.value = content;
            this.currentContent = content;
            this.originalContent = content;
            
            globalState.hasUnsavedChanges = false;
            this.updateSaveButtonState();
            
        } catch (error) {
            console.error('Failed to load profile content:', error);
            showToast(translations[currentLanguage].save_error, 'error');
            this.jsonEditor.value = '';
        } finally {
            showLoading(false);
        }
    }
    
    showEditorContent() {
        console.log('Showing editor content');
        this.jsonEditorView.style.display = 'flex';
        this.aboutView.style.display = 'none';
    }
    
    showAboutContent() {
        console.log('Showing about content');
        this.jsonEditorView.style.display = 'none';
        this.aboutView.style.display = 'flex';
        this.profileTitle.textContent = translations[currentLanguage].about_nav_item;
    }
    
    handleContentChange() {
        this.currentContent = this.jsonEditor.value;
        globalState.hasUnsavedChanges = this.currentContent !== this.originalContent;
        this.updateSaveButtonState();
    }
    
    updateSaveButtonState() {
        this.saveButton.disabled = !globalState.hasUnsavedChanges || globalState.isLoading;
        this.saveAsButton.disabled = globalState.isLoading;
    }
    
    async save() {
        if (!globalState.currentProfile || globalState.currentProfile === 'about') {
            return;
        }
        
        try {
            showLoading(true, translations[currentLanguage].saving_profile);
            
            // Validate JSON
            const validationResult = await invoke('validate_json_content', { 
                content: this.currentContent 
            });
            
            if (!validationResult.is_valid) {
                const errorMessages = validationResult.errors.map(e => 
                    `Line ${e.line}, Column ${e.column}: ${e.message}`
                ).join('\n');
                this.showCustomAlert(`${translations[currentLanguage].validation_error}:\n${errorMessages}`);
                return;
            }
            
            // Save profile
            await invoke('save_profile', {
                profileId: globalState.currentProfile,
                content: this.currentContent
            });
            
            this.originalContent = this.currentContent;
            globalState.hasUnsavedChanges = false;
            this.updateSaveButtonState();
            
            showToast(translations[currentLanguage].save_success, 'success');
            
        } catch (error) {
            console.error('Failed to save profile:', error);
            showToast(translations[currentLanguage].save_error, 'error');
        } finally {
            showLoading(false);
        }
    }
    
    async showSaveAsModal() {
        // 先验证JSON格式
        try {
            const validationResult = await invoke('validate_json_content', { 
                content: this.currentContent 
            });
            
            if (!validationResult.is_valid) {
                const errorMessages = validationResult.errors.map(e => 
                    `Line ${e.line}, Column ${e.column}: ${e.message}`
                ).join('\n');
                this.showCustomAlert(`${translations[currentLanguage].validation_error}:\n${errorMessages}`);
                return;
            }
            
            // JSON格式正确，显示另存为对话框
            if (window.saveAsModal) {
                window.saveAsModal.show(this.currentContent);
            }
        } catch (error) {
            console.error('Failed to validate JSON before save as:', error);
            this.showCustomAlert(translations[currentLanguage].validation_error);
        }
    }
    
    // 自定义警告对话框，使用系统图标
    showCustomAlert(message) {
        // 创建自定义对话框
        const alertModal = document.createElement('div');
        alertModal.className = 'modal-overlay';
        alertModal.style.display = 'flex';
        
        alertModal.innerHTML = `
            <div class="modal-content alert-modal">
                <div class="modal-header">
                    <div class="alert-icon">⚠️</div>
                    <h3>${translations[currentLanguage].validation_error || 'Validation Error'}</h3>
                </div>
                <div class="modal-body">
                    <pre class="error-message">${message}</pre>
                </div>
                <div class="modal-footer">
                    <button class="primary-button alert-ok-button">OK</button>
                </div>
            </div>
        `;
        
        document.body.appendChild(alertModal);
        
        // 添加事件监听
        const okButton = alertModal.querySelector('.alert-ok-button');
        okButton.addEventListener('click', () => {
            document.body.removeChild(alertModal);
        });
        
        // ESC键关闭
        const handleEsc = (e) => {
            if (e.key === 'Escape') {
                document.body.removeChild(alertModal);
                document.removeEventListener('keydown', handleEsc);
            }
        };
        document.addEventListener('keydown', handleEsc);
        
        // 点击背景关闭
        alertModal.addEventListener('click', (e) => {
            if (e.target === alertModal) {
                document.body.removeChild(alertModal);
                document.removeEventListener('keydown', handleEsc);
            }
        });
    }
}

// Save As Modal Component
class SaveAsModal {
    constructor() {
        this.modal = document.getElementById('save-as-modal');
        this.profileNameInput = document.getElementById('profile-name-input');
        this.filenamePreview = document.getElementById('filename-preview');
        this.cancelButton = document.getElementById('modal-cancel-button');
        this.saveButton = document.getElementById('modal-save-button');
        this.closeButton = document.getElementById('modal-close-button');
        
        this.currentContent = '';
        
        this.initializeEventListeners();
    }
    
    initializeEventListeners() {
        // Profile name input
        this.profileNameInput.addEventListener('input', () => {
            this.updateFilenamePreview();
            this.validateInput();
        });
        
        // Modal buttons
        this.cancelButton.addEventListener('click', () => this.hide());
        this.closeButton.addEventListener('click', () => this.hide());
        this.saveButton.addEventListener('click', () => this.handleSave());
        
        // Close on overlay click
        this.modal.addEventListener('click', (e) => {
            if (e.target === this.modal) {
                this.hide();
            }
        });
        
        // ESC key
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && this.modal.style.display !== 'none') {
                this.hide();
            }
        });
    }
    
    show(content) {
        this.currentContent = content;
        this.profileNameInput.value = '';
        this.updateFilenamePreview();
        this.validateInput();
        this.modal.style.display = 'flex';
        this.profileNameInput.focus();
    }
    
    hide() {
        this.modal.style.display = 'none';
        this.profileNameInput.value = '';
    }
    
    updateFilenamePreview() {
        const name = this.profileNameInput.value.trim();
        if (name) {
            this.filenamePreview.textContent = `${name}.settings.json`;
        } else {
            this.filenamePreview.textContent = 'profile-name.settings.json';
        }
    }
    
    validateInput() {
        const name = this.profileNameInput.value.trim();
        const isValid = name.length > 0 && !/[\/\\:*?"<>|]/.test(name);
        
        this.saveButton.disabled = !isValid;
        
        // Update input style
        if (name.length > 0) {
            this.profileNameInput.classList.toggle('invalid', !isValid);
        } else {
            this.profileNameInput.classList.remove('invalid');
        }
    }
    
    async handleSave() {
        const profileName = this.profileNameInput.value.trim();
        
        if (!profileName) {
            return;
        }
        
        try {
            showLoading(true, translations[currentLanguage].saving_profile);
            
            const filePath = await invoke('create_new_profile', {
                profileName,
                content: this.currentContent
            });
            
            this.hide();
            showToast(translations[currentLanguage].profile_created, 'success');
            
            // Refresh navigation panel
            if (window.navigationPanel) {
                window.navigationPanel.refreshProfileList();
            }
            
        } catch (error) {
            console.error('Failed to create profile:', error);
            showToast(`${translations[currentLanguage].profile_create_error}: ${error}`, 'error');
        } finally {
            showLoading(false);
        }
    }
}

function updateProfilesCount() {
    const profilesCountElement = document.getElementById('profiles-count');
    if (profilesCountElement && globalState.profiles) {
        profilesCountElement.textContent = globalState.profiles.length.toString();
    }
}

// Initialize application
function initializeApp() {
    console.log('Initializing enhanced CCCS settings...');
    
    // Initialize Tauri APIs
    if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.window) {
        invoke = window.__TAURI__.core.invoke;
        getCurrentWindow = window.__TAURI__.window.getCurrentWindow;
        console.log('Tauri APIs initialized successfully');
    } else {
        console.error('Tauri APIs not available');
        return;
    }
    
    // Debug: 检查当前窗口尺寸
    console.log('Current window size:', window.innerWidth, 'x', window.innerHeight);
    console.log('Screen size:', window.screen.width, 'x', window.screen.height);
    
    // Set window size programmatically
    if (getCurrentWindow) {
        
        // 先获取当前尺寸
        getCurrentWindow().innerSize()
            .then(size => {
                console.log('Tauri window size:', size.width, 'x', size.height);
                
                // 尝试设置新尺寸
                return getCurrentWindow().setSize(size.type === 'Physical' ? 
                    {type: 'Physical', width: 1400, height: 1300} : 
                    {type: 'Logical', width: 1400, height: 1300}
                );
            })
            .then(() => {
                console.log('Window resized to 1400x1300');
                // 验证是否成功
                return getCurrentWindow().innerSize();
            })
            .then(newSize => {
                console.log('New window size:', newSize.width, 'x', newSize.height);
            })
            .catch(e => console.log('Window resize failed:', e));
    }
    
    // Detect and set language
    currentLanguage = detectSystemLanguage();
    updateTexts();
    
    // Initialize components
    window.navigationPanel = new NavigationPanel();
    window.contentEditor = new ContentEditor();
    window.saveAsModal = new SaveAsModal();
    
    // Debug: 检查CSS是否正确应用
    setTimeout(() => {
        const jsonEditor = document.getElementById('json-editor');
        const editorControls = document.querySelector('.editor-controls');
        
        if (jsonEditor) {
            const style = window.getComputedStyle(jsonEditor);
            console.log('JSON Editor computed style:');
            console.log('- flex:', style.flex);
            console.log('- height:', style.height);
            console.log('- padding-bottom:', style.paddingBottom);
        }
        
        if (editorControls) {
            const style = window.getComputedStyle(editorControls);
            console.log('Editor Controls computed style:');
            console.log('- position:', style.position);
            console.log('- bottom:', style.bottom);
            console.log('- right:', style.right);
        }
    }, 2000);
    
    // Load initial data
    window.navigationPanel.loadProfiles();
    
    // Set up language selector
    const languageSelect = document.getElementById('language-select');
    if (languageSelect) {
        languageSelect.addEventListener('change', (e) => {
            const selectedLang = e.target.value;
            if (selectedLang) {
                currentLanguage = selectedLang;
            } else {
                currentLanguage = detectSystemLanguage();
            }
            updateTexts();
            // Optionally save language preference here
        });
    }
    
    // Set up close button for About section
    const closeButton = document.getElementById('close-button');
    if (closeButton) {
        closeButton.addEventListener('click', async () => {
            try {
                await invoke('close_settings_window');
            } catch (error) {
                console.error('Failed to close window:', error);
            }
        });
    }
    
    // Update profiles count in About section
    updateProfilesCount();
    
    console.log('Enhanced CCCS settings initialized successfully');
}

// Start the application when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeApp);
} else {
    initializeApp();
}