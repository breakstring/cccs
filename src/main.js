// CCCS Enhanced Settings Page - Main Entry Point
import "./style.css";

// Global state
let globalState = {
  currentProfile: null,
  profiles: [],
  isLoading: false,
  hasUnsavedChanges: false,
  lastError: null,
};

// Tauri APIs - will be initialized when available
let invoke = null;
let getCurrentWindow = null;

// Internationalization (enhanced with new keys)
const translations = {
  en: {
    // Existing translations
    app_name: "CCCS",
    settings_title: "CCCS - Settings",
    app_info_title: "Application Information",
    app_description:
      "CCCS (Claude Code Configuration Switcher) is a tool for quickly switching Claude Code configuration files.",
    status_icons_title: "Profile Status Icons",
    full_match_description:
      "Complete match - configuration fully matches current settings",
    partial_match_description:
      "Partial match - identical except model field (auto-updated by Claude Code)",
    error_status_description:
      "Error - failed to read or parse configuration file",
    no_match_description:
      "No icon - configuration differs from current settings",
    language_settings_title: "Language Settings",
    interface_language_label: "Interface language:",
    follow_system: "Follow system",
    chinese: "中文",
    english: "English",
    current_status_title: "Current Status",
    claude_directory_label: "Claude Code directory:",
    profiles_found_label: "Configuration files found:",
    close_button: "Close",
    minimize_button: "Minimize",
    exit_button: "Exit",
    saving_settings: "Saving settings...",

    // New translations for enhanced UI
    profiles_section_title: "Profiles",
    settings_nav_item: "Settings",
    current_profile_title: "Current",
    current_profile_display_name: "Current",
    json_editor_title: "Configuration Editor",
    json_editor_hint: "Edit your configuration in JSON format",
    json_editor_placeholder: "Loading configuration...",
    save_button: "Save",
    apply_profile_button: "Use This Profile",
    save_as_button: "Save As...",
    delete_button: "Delete Profile",
    save_as_modal_title: "Save As New Profile",
    profile_name_label: "Profile Name:",
    profile_name_placeholder: "Enter profile name",
    profile_name_hint:
      'Profile names cannot contain special characters: / \\ : * ? " < > | or start/end with spaces or dots',
    preview_filename_label: "Preview:",
    cancel_button: "Cancel",
    delete_confirm_title: "Confirm Delete",
    delete_confirm_message:
      "Are you sure you want to delete this profile? This action cannot be undone.",
    delete_button_confirm: "Delete",
    unsaved_changes_title: "Unsaved Changes",
    unsaved_changes_message:
      "You have unsaved changes. Do you want to discard them and continue?",
    discard_button: "Discard Changes",

    // Status messages
    loading_profiles: "Loading profiles...",
    loading_content: "Loading content...",
    saving_profile: "Saving profile...",
    save_success: "Profile saved successfully",
    save_error: "Failed to save profile",
    validation_error: "Invalid JSON format",
    profile_created: "Profile created successfully",
    profile_create_error: "Failed to create profile",
    profile_deleted: "Profile deleted successfully",
    profile_delete_error: "Failed to delete profile",

    // Field exclusion settings
    field_exclusion_title: "Field Exclusion Settings",
    field_exclusion_description: "Configure which fields to ignore when comparing profiles with current settings.",
    ignored_fields_label: "Ignored Fields:",
    add_field_button: "Add Field",
    field_name_placeholder: "Enter field name",
    confirm_button: "✓",
    cancel_field_button: "✗",
    reset_to_default_button: "Reset to Default",
    save_settings_button: "Save Settings",
    status_icons_description: "Understanding profile comparison results:",
    full_match_title: "Complete Match",
    partial_match_title: "Partial Match",
    error_status_title: "Error",
    no_match_title: "No Match",

    // Field validation messages
    field_name_empty: "Field name cannot be empty",
    field_name_invalid: "Field name contains invalid characters",
    field_name_too_long: "Field name too long (max 100 characters)",
    field_already_exists: "Field already exists",
    settings_saved: "Settings saved successfully",
    settings_save_error: "Failed to save settings",
  },
  zh: {
    // Existing translations
    app_name: "CCCS",
    settings_title: "CCCS - 设置",
    app_info_title: "应用程序信息",
    app_description:
      "CCCS (Claude Code Configuration Switcher) 是一个用于快速切换 Claude Code 配置文件的工具。",
    status_icons_title: "配置状态图标",
    full_match_description: "完全匹配 - 配置与当前设置完全一致",
    partial_match_description:
      "部分匹配 - 除model字段外完全一致（Claude Code自动更新）",
    error_status_description: "错误 - 读取或解析配置文件失败",
    no_match_description: "无图标 - 配置与当前设置不同",
    language_settings_title: "语言设置",
    interface_language_label: "界面语言:",
    follow_system: "跟随系统",
    chinese: "中文",
    english: "English",
    current_status_title: "当前状态",
    claude_directory_label: "Claude Code 目录:",
    profiles_found_label: "发现的配置文件:",
    close_button: "关闭",
    minimize_button: "最小化",
    exit_button: "退出",
    saving_settings: "正在保存设置...",

    // New translations for enhanced UI
    profiles_section_title: "配置文件",
    settings_nav_item: "设置",
    current_profile_title: "当前配置",
    current_profile_display_name: "当前配置",
    json_editor_title: "配置编辑器",
    json_editor_hint: "以JSON格式编辑您的配置",
    json_editor_placeholder: "正在加载配置...",
    save_button: "保存",
    apply_profile_button: "使用该配置",
    save_as_button: "另存为...",
    delete_button: "删除配置文件",
    save_as_modal_title: "另存为新的配置文件",
    profile_name_label: "配置文件名:",
    profile_name_placeholder: "输入配置文件名",
    profile_name_hint: '配置文件名不能包含特殊字符: / \\ : * ? " < > | 或以空格、点开头/结尾',
    preview_filename_label: "预览:",
    cancel_button: "取消",
    delete_confirm_title: "确认删除",
    delete_confirm_message: "您确定要删除这个配置文件吗？此操作无法撤销。",
    delete_button_confirm: "删除",
    unsaved_changes_title: "未保存的更改",
    unsaved_changes_message: "您有未保存的更改。是否要放弃这些更改并继续？",
    discard_button: "放弃更改",

    // Status messages
    loading_profiles: "正在加载配置文件...",
    loading_content: "正在加载内容...",
    saving_profile: "正在保存配置文件...",
    save_success: "配置文件保存成功",
    save_error: "保存配置文件失败",
    validation_error: "JSON格式无效",
    profile_created: "配置文件创建成功",
    profile_create_error: "创建配置文件失败",
    profile_deleted: "配置文件删除成功",
    profile_delete_error: "删除配置文件失败",

    // Field exclusion settings
    field_exclusion_title: "字段排除设置",
    field_exclusion_description: "配置在比较配置文件与当前设置时要忽略的字段。",
    ignored_fields_label: "忽略的字段:",
    add_field_button: "添加字段",
    field_name_placeholder: "输入字段名",
    confirm_button: "✓",
    cancel_field_button: "✗",
    reset_to_default_button: "重置为默认",
    save_settings_button: "保存设置",
    status_icons_description: "理解配置比较结果:",
    full_match_title: "完全匹配",
    partial_match_title: "部分匹配",
    error_status_title: "错误",
    no_match_title: "不匹配",

    // Field validation messages
    field_name_empty: "字段名不能为空",
    field_name_invalid: "字段名包含无效字符",
    field_name_too_long: "字段名过长（最大100字符）",
    field_already_exists: "字段已存在",
    settings_saved: "设置保存成功",
    settings_save_error: "保存设置失败",
  },
};

let currentLanguage = "en";

// Utility functions
function detectSystemLanguage() {
  // Get user's preferred languages in order
  const languages = navigator.languages || [navigator.language || navigator.userLanguage || 'en'];
  
  for (const lang of languages) {
    if (lang) {
      const langCode = lang.toLowerCase();
      
      // Support various Chinese language codes and regions
      if (langCode.startsWith('zh') || 
          langCode.includes('chinese') ||
          langCode === 'cn' ||
          langCode.startsWith('zh-cn') ||    // Simplified Chinese (China)
          langCode.startsWith('zh-sg') ||    // Simplified Chinese (Singapore)
          langCode.startsWith('zh-tw') ||    // Traditional Chinese (Taiwan)
          langCode.startsWith('zh-hk') ||    // Traditional Chinese (Hong Kong)
          langCode.startsWith('zh-mo') ||    // Traditional Chinese (Macau)
          langCode.startsWith('zh-hans') ||  // Simplified Chinese
          langCode.startsWith('zh-hant')) {  // Traditional Chinese
        return "zh";
      }
      
      // Support English variants
      if (langCode.startsWith('en')) {
        return "en";
      }
    }
  }
  
  // Fallback to English if no supported language found
  return "en";
}

function updateTexts() {
  const t = translations[currentLanguage];

  document.querySelectorAll("[data-i18n]").forEach((element) => {
    const key = element.getAttribute("data-i18n");
    if (t[key]) {
      element.textContent = t[key];
    }
  });

  // Update placeholders
  document.querySelectorAll("[data-i18n-placeholder]").forEach((element) => {
    const key = element.getAttribute("data-i18n-placeholder");
    if (t[key]) {
      element.placeholder = t[key];
    }
  });

  // Update page title
  document.title = t.settings_title;
}

function showToast(message, type = "info") {
  const toastContainer = document.getElementById("toast-container");
  const toast = document.createElement("div");
  toast.className = `toast toast-${type}`;
  toast.textContent = message;

  toastContainer.appendChild(toast);

  // Auto remove after 3 seconds
  setTimeout(() => {
    toast.remove();
  }, 3000);
}

function showLoading(show, message = null) {
  const overlay = document.getElementById("loading-overlay");
  const text = overlay.querySelector(".loading-text");

  if (show) {
    if (message) {
      text.textContent = message;
    }
    overlay.style.display = "flex";
  } else {
    overlay.style.display = "none";
  }

  globalState.isLoading = show;
}

// Navigation Panel Component
class NavigationPanel {
  constructor() {
    this.selectedItem = null;
    this.profileNavList = document.getElementById("profile-nav-list");
    this.settingsNavItem = document.querySelector(".settings-nav-item");

    this.initializeEventListeners();
  }

  initializeEventListeners() {
    // Settings navigation click
    this.settingsNavItem.addEventListener("click", () => {
      this.handleItemClick("settings");
    });
  }

  async loadProfiles(shouldAutoSelect = true) {
    console.log("NavigationPanel.loadProfiles: Starting to load profiles, shouldAutoSelect:", shouldAutoSelect);
    try {
      showLoading(true, translations[currentLanguage].loading_profiles);

      console.log("NavigationPanel.loadProfiles: Invoking get_profiles_list");
      const profiles = await invoke("get_profiles_list");
      console.log("NavigationPanel.loadProfiles: Received profiles:", profiles.length, "profiles");
      
      globalState.profiles = profiles;

      this.renderProfileList(profiles);
      console.log("NavigationPanel.loadProfiles: Rendered profile list");

      // 只在初次加载时自动选择第一个配置，刷新时保持当前配置
      if (shouldAutoSelect && profiles.length > 0) {
        console.log("NavigationPanel.loadProfiles: Auto-selecting first profile:", profiles[0].id);
        // 修复竞态条件：直接在loading结束后切换，不使用setTimeout
        this.switchToProfile(profiles[0].id);
      } else if (!shouldAutoSelect && globalState.currentProfile) {
        console.log("NavigationPanel.loadProfiles: Maintaining current profile:", globalState.currentProfile);
        // 刷新时保持当前选中的配置，更新导航状态
        this.updateActiveState(globalState.currentProfile);
        // 确保内容也被重新加载
        if (window.contentEditor) {
          console.log("NavigationPanel.loadProfiles: Reloading current profile content after refresh");
          window.contentEditor.loadProfile(globalState.currentProfile);
        }
      } else {
        console.log("NavigationPanel.loadProfiles: No auto-selection, no current profile to maintain");
      }

      // Update profiles count in About section
      updateProfilesCount();
      console.log("NavigationPanel.loadProfiles: Updated profiles count");
    } catch (error) {
      console.error("NavigationPanel.loadProfiles: Failed to load profiles:", error);
      // 不在这里显示错误通知，避免与其他操作的成功通知冲突
      // showToast("Failed to load profiles", "error");
    } finally {
      // 确保在所有操作完成后才结束loading状态
      showLoading(false);
      console.log("NavigationPanel.loadProfiles: Finished loading profiles");
    }
  }

  renderProfileList(profiles) {
    this.profileNavList.innerHTML = "";

    profiles.forEach((profile) => {
      const listItem = document.createElement("li");
      listItem.className = "nav-item profile-nav-item";
      listItem.setAttribute("data-profile-id", profile.id);

      // Add special class for Current profile
      if (profile.id === "current") {
        listItem.classList.add("current-profile");
      }

      // Create icon span for status
      const iconSpan = document.createElement("span");
      iconSpan.className = "nav-item-icon";

      // Create text span
      const text = document.createElement("span");
      text.className = "nav-item-text";
      
      // Use localized display name for Current profile
      if (profile.id === "current") {
        text.textContent = translations[currentLanguage].current_profile_display_name;
      } else {
        text.textContent = profile.display_name;
      }

      listItem.appendChild(iconSpan);
      listItem.appendChild(text);

      listItem.addEventListener("click", () => {
        this.handleItemClick(profile.id);
      });

      this.profileNavList.appendChild(listItem);
    });

    // Load status icons after rendering
    this.loadProfileStatuses();
  }

  handleItemClick(profileId) {
    // 如果有未保存的更改，显示确认对话框
    if (
      globalState.hasUnsavedChanges &&
      globalState.currentProfile &&
      globalState.currentProfile !== profileId
    ) {
      this.showUnsavedChangesConfirm(profileId);
      return;
    }

    this.switchToProfile(profileId);
  }

  showUnsavedChangesConfirm(targetProfileId) {
    const t = translations[currentLanguage];

    // 创建确认对话框
    const confirmModal = document.createElement("div");
    confirmModal.className = "modal-overlay";
    confirmModal.style.display = "flex";

    confirmModal.innerHTML = `
            <div class="modal-content delete-confirm-modal">
                <div class="modal-header">
                    <div class="alert-icon">⚠️</div>
                    <h3>${t.unsaved_changes_title}</h3>
                </div>
                <div class="modal-body">
                    <p>${t.unsaved_changes_message}</p>
                </div>
                <div class="modal-footer">
                    <button class="secondary-button unsaved-cancel-button">${t.cancel_button}</button>
                    <button class="danger-button unsaved-discard-button">${t.discard_button}</button>
                </div>
            </div>
        `;

    document.body.appendChild(confirmModal);

    // 添加事件监听
    const cancelButton = confirmModal.querySelector(".unsaved-cancel-button");
    const discardButton = confirmModal.querySelector(".unsaved-discard-button");

    const closeModal = () => {
      document.body.removeChild(confirmModal);
      document.removeEventListener("keydown", handleEsc);
    };

    cancelButton.addEventListener("click", closeModal);

    discardButton.addEventListener("click", () => {
      closeModal();
      // 放弃更改，切换到目标配置
      this.switchToProfile(targetProfileId);
    });

    // ESC键关闭
    const handleEsc = (e) => {
      if (e.key === "Escape") {
        closeModal();
      }
    };
    document.addEventListener("keydown", handleEsc);

    // 点击背景关闭
    confirmModal.addEventListener("click", (e) => {
      if (e.target === confirmModal) {
        closeModal();
      }
    });
  }

  switchToProfile(profileId) {
    console.log("NavigationPanel.switchToProfile: Switching to profile:", profileId, "from:", globalState.currentProfile);
    
    // 先更新全局状态
    globalState.currentProfile = profileId;
    globalState.hasUnsavedChanges = false;
    console.log("NavigationPanel.switchToProfile: Updated global state");

    // 更新导航栏的激活状态
    this.updateActiveState(profileId);
    console.log("NavigationPanel.switchToProfile: Updated navigation active state");

    // 通知内容编辑器加载配置
    if (window.contentEditor) {
      console.log("NavigationPanel.switchToProfile: Calling contentEditor.loadProfile");
      window.contentEditor.loadProfile(profileId);
    } else {
      console.warn("NavigationPanel.switchToProfile: window.contentEditor not available");
    }
  }

  updateActiveState(profileId) {
    // Remove active class from all items
    document.querySelectorAll(".nav-item").forEach((item) => {
      item.classList.remove("active");
    });

    // Add active class to selected item
    if (profileId === "settings") {
      // Handle settings navigation item
      const settingsItem = document.querySelector(".settings-nav-item");
      if (settingsItem) {
        settingsItem.classList.add("active");
        this.selectedItem = profileId;
      }
    } else {
      // Handle profile navigation items
      const selectedItem = document.querySelector(
        `[data-profile-id="${profileId}"]`
      );
      if (selectedItem) {
        selectedItem.classList.add("active");
        this.selectedItem = profileId;
      }
    }
  }

  async refreshProfileList() {
    // 刷新时不自动切换配置，保持用户当前查看的配置
    return this.loadProfiles(false);
  }

  // Load profile status icons
  async loadProfileStatuses() {
    try {
      const profiles = globalState.profiles;

      for (const profile of profiles) {
        if (profile.id === "current") {
          continue; // Skip current profile
        }

        // Get profile status using the new command
        const statusIcon = await invoke("get_profile_status", {
          profileId: profile.id,
        });

        // Update the icon in the UI
        const listItem = document.querySelector(
          `[data-profile-id="${profile.id}"]`
        );
        if (listItem) {
          const iconSpan = listItem.querySelector(".nav-item-icon");
          if (iconSpan) {
            iconSpan.textContent = statusIcon;
          }
        }
      }
    } catch (error) {
      console.error("Failed to load profile statuses:", error);
    }
  }
}

// Content Editor Component
class ContentEditor {
  constructor() {
    this.profileTitle = document.getElementById("profile-title");
    this.jsonEditor = document.getElementById("json-editor");
    this.jsonEditorView = document.getElementById("json-editor-view");
    this.settingsView = document.getElementById("settings-view");
    this.saveButton = document.getElementById("save-button");
    this.applyProfileButton = document.getElementById("apply-profile-button");
    this.saveAsButton = document.getElementById("save-as-button");
    this.deleteButton = document.getElementById("delete-button");

    this.currentContent = "";
    this.originalContent = "";

    this.initializeEventListeners();
  }

  initializeEventListeners() {
    // Content change tracking
    this.jsonEditor.addEventListener("input", () => {
      this.handleContentChange();
    });

    // Save button
    this.saveButton.addEventListener("click", () => {
      this.save();
    });

    // Apply Profile button
    this.applyProfileButton.addEventListener("click", () => {
      this.applyProfile();
    });

    // Save As button
    this.saveAsButton.addEventListener("click", () => {
      this.showSaveAsModal();
    });

    // Delete button
    this.deleteButton.addEventListener("click", () => {
      this.showDeleteConfirmModal();
    });
  }

  async loadProfile(profileId) {
    console.log("ContentEditor.loadProfile: Starting to load profile:", profileId);
    try {
      showLoading(true, translations[currentLanguage].loading_content);

      if (profileId === "settings") {
        console.log("ContentEditor.loadProfile: Loading settings content");
        this.showSettingsContent();
        return;
      }

      console.log("ContentEditor.loadProfile: Showing editor content");
      this.showEditorContent();

      // Update title
      const profile = globalState.profiles.find((p) => p.id === profileId);
      if (profile) {
        console.log("ContentEditor.loadProfile: Found profile, updating title to:", profile.display_name);
        
        // Use localized title for Current profile
        if (profileId === "current") {
          this.profileTitle.textContent = translations[currentLanguage].current_profile_display_name;
        } else {
          this.profileTitle.textContent = profile.display_name;
        }
      } else {
        console.warn("ContentEditor.loadProfile: Profile not found in globalState.profiles for id:", profileId);
      }

      // 显示或隐藏删除按钮（Current配置文件不显示删除按钮）
      if (this.deleteButton) {
        if (profileId === "current") {
          console.log("ContentEditor.loadProfile: Hiding delete button for current profile");
          this.deleteButton.style.display = "none";
        } else {
          console.log("ContentEditor.loadProfile: Showing delete button for profile:", profileId);
          this.deleteButton.style.display = "inline-flex";
        }
      }

      // 显示或隐藏应用配置按钮（Current和Settings不显示应用按钮）
      if (this.applyProfileButton) {
        if (profileId === "current" || profileId === "settings") {
          console.log("ContentEditor.loadProfile: Hiding apply button for profile:", profileId);
          this.applyProfileButton.style.display = "none";
        } else {
          console.log("ContentEditor.loadProfile: Showing apply button for profile:", profileId);
          this.applyProfileButton.style.display = "inline-flex";
        }
      }

      // Load content
      console.log("ContentEditor.loadProfile: Invoking load_profile_content for:", profileId);
      const content = await invoke("load_profile_content", { profileId });
      console.log("ContentEditor.loadProfile: Received content for", profileId, "length:", content.length);
      
      this.jsonEditor.value = content;
      this.currentContent = content;
      this.originalContent = content;

      globalState.hasUnsavedChanges = false;
      console.log("ContentEditor.loadProfile: Successfully loaded profile content for:", profileId);
    } catch (error) {
      console.error("ContentEditor.loadProfile: Failed to load profile content for", profileId, "error:", error);
      // 不显示错误通知，避免与其他操作的通知冲突
      // showToast("Failed to load profile content", "error");
      this.jsonEditor.value = "";
    } finally {
      showLoading(false);
      console.log("ContentEditor.loadProfile: Finished loading profile, updating button state");
      // 使用 setTimeout 确保状态更新在下一个事件循环中执行
      setTimeout(() => {
        this.updateSaveButtonState();
      }, 0);
    }
  }

  showEditorContent() {
    console.log("Showing editor content");
    this.jsonEditorView.style.display = "flex";
    this.settingsView.style.display = "none";

    // 按钮状态将在loadProfile完成后统一更新，避免竞争条件
  }

  showSettingsContent() {
    console.log("Showing settings content");
    this.jsonEditorView.style.display = "none";
    this.settingsView.style.display = "flex";
    this.profileTitle.textContent =
      translations[currentLanguage].settings_nav_item;

    // Initialize settings view
    if (window.settingsManager) {
      window.settingsManager.loadSettings();
    }

    // Settings页面不需要更新编辑按钮状态，因为编辑按钮区域已经隐藏了
  }

  handleContentChange() {
    // 只有在有效配置文件时才处理内容变化
    if (!globalState.currentProfile || globalState.currentProfile === "settings") {
      return;
    }

    this.currentContent = this.jsonEditor.value;
    globalState.hasUnsavedChanges =
      this.currentContent !== this.originalContent;
    this.updateSaveButtonState();
  }

  updateSaveButtonState() {
    // 基本可用性：按钮在非加载状态下默认可用
    const isLoadingState = globalState.isLoading;
    const isValidProfile =
      globalState.currentProfile && globalState.currentProfile !== "settings";

    // 保存和另存为按钮：在有效配置且非加载状态下可用
    this.saveButton.disabled = isLoadingState || !isValidProfile;
    this.saveAsButton.disabled = isLoadingState || !isValidProfile;

    // 删除按钮：只对非Current配置文件且非加载状态下可用
    const deleteButton = document.getElementById("delete-button");
    if (deleteButton) {
      const canDelete =
        isValidProfile && globalState.currentProfile !== "current";
      deleteButton.disabled = isLoadingState || !canDelete;
    }

    console.log("Button state updated:", {
      isLoading: isLoadingState,
      currentProfile: globalState.currentProfile,
      saveDisabled: this.saveButton.disabled,
      saveAsDisabled: this.saveAsButton.disabled,
      deleteDisabled: deleteButton ? deleteButton.disabled : "N/A",
    });
  }

  async save() {
    if (!globalState.currentProfile || globalState.currentProfile === "settings") {
      return;
    }

    // 检查配置是否还存在（防止在删除后触发保存）
    const profileExists = globalState.profiles.some(
      (p) => p.id === globalState.currentProfile
    );
    if (!profileExists && globalState.currentProfile !== "current") {
      console.warn(
        "Attempted to save non-existent profile:",
        globalState.currentProfile
      );
      return;
    }

    try {
      showLoading(true, translations[currentLanguage].saving_profile);

      // Validate JSON
      const validationResult = await invoke("validate_json_content", {
        content: this.currentContent,
      });

      if (!validationResult.is_valid) {
        const errorMessages = validationResult.errors
          .map((e) => `Line ${e.line}, Column ${e.column}: ${e.message}`)
          .join("\n");
        this.showCustomAlert(
          `${translations[currentLanguage].validation_error}:\n${errorMessages}`
        );
        return;
      }

      // Save profile
      await invoke("save_profile", {
        profileId: globalState.currentProfile,
        content: this.currentContent,
      });

      this.originalContent = this.currentContent;
      globalState.hasUnsavedChanges = false;

      showToast(translations[currentLanguage].save_success, "success");

      // 延迟刷新状态图标，避免与通知冲突
      setTimeout(() => {
        if (window.navigationPanel) {
          window.navigationPanel.loadProfileStatuses();
        }
      }, 300);
    } catch (error) {
      console.error("Failed to save profile:", error);
      showToast(translations[currentLanguage].save_error, "error");
    } finally {
      showLoading(false);
      // 按钮状态会在加载状态结束后自动更新
    }
  }

  async applyProfile() {
    if (!globalState.currentProfile || globalState.currentProfile === "settings") {
      return;
    }

    // 不能应用"current"配置到自己
    if (globalState.currentProfile === "current") {
      showToast("Cannot apply current configuration to itself", "error");
      return;
    }

    // 检查配置是否还存在
    const profileExists = globalState.profiles.some(
      (p) => p.id === globalState.currentProfile
    );
    if (!profileExists) {
      console.warn(
        "Attempted to apply non-existent profile:",
        globalState.currentProfile
      );
      showToast("Profile not found", "error");
      return;
    }

    try {
      showLoading(true, "Applying profile...");

      // 先验证JSON格式
      const validationResult = await invoke("validate_json_content", {
        content: this.currentContent,
      });

      if (!validationResult.is_valid) {
        const errorMessages = validationResult.errors
          .map((e) => `Line ${e.line}, Column ${e.column}: ${e.message}`)
          .join("\n");
        this.showCustomAlert(
          `${translations[currentLanguage].validation_error}:\n${errorMessages}`
        );
        return;
      }

      // 如果有未保存的更改，先保存
      if (globalState.hasUnsavedChanges) {
        await invoke("save_profile", {
          profileId: globalState.currentProfile,
          content: this.currentContent,
        });
        this.originalContent = this.currentContent;
        globalState.hasUnsavedChanges = false;
      }

      // 应用配置文件
      await invoke("apply_profile", {
        profileId: globalState.currentProfile,
      });

      showToast("Profile applied successfully!", "success");

      // 延迟刷新状态图标
      setTimeout(() => {
        if (window.navigationPanel) {
          window.navigationPanel.loadProfileStatuses();
        }
      }, 300);
    } catch (error) {
      console.error("Failed to apply profile:", error);
      showToast(`Failed to apply profile: ${error}`, "error");
    } finally {
      showLoading(false);
    }
  }

  async showSaveAsModal() {
    // 先验证JSON格式
    try {
      const validationResult = await invoke("validate_json_content", {
        content: this.currentContent,
      });

      if (!validationResult.is_valid) {
        const errorMessages = validationResult.errors
          .map((e) => `Line ${e.line}, Column ${e.column}: ${e.message}`)
          .join("\n");
        this.showCustomAlert(
          `${translations[currentLanguage].validation_error}:\n${errorMessages}`
        );
        return;
      }

      // JSON格式正确，显示另存为对话框
      if (window.saveAsModal) {
        window.saveAsModal.show(this.currentContent);
      }
    } catch (error) {
      console.error("Failed to validate JSON before save as:", error);
      this.showCustomAlert(translations[currentLanguage].validation_error);
    }
  }

  showDeleteConfirmModal() {
    if (
      !globalState.currentProfile ||
      globalState.currentProfile === "current" ||
      globalState.currentProfile === "settings"
    ) {
      return;
    }

    const profile = globalState.profiles.find(
      (p) => p.id === globalState.currentProfile
    );
    const profileName = profile
      ? profile.display_name
      : globalState.currentProfile;

    // 创建删除确认对话框
    const deleteModal = document.createElement("div");
    deleteModal.className = "modal-overlay";
    deleteModal.style.display = "flex";

    deleteModal.innerHTML = `
            <div class="modal-content delete-confirm-modal">
                <div class="modal-header">
                    <div class="alert-icon">⚠️</div>
                    <h3>${translations[currentLanguage].delete_confirm_title}</h3>
                </div>
                <div class="modal-body">
                    <p>${translations[currentLanguage].delete_confirm_message}</p>
                    <p><strong>${profileName}</strong></p>
                </div>
                <div class="modal-footer">
                    <button class="secondary-button delete-cancel-button">${translations[currentLanguage].cancel_button}</button>
                    <button class="danger-button delete-confirm-button">${translations[currentLanguage].delete_button_confirm}</button>
                </div>
            </div>
        `;

    document.body.appendChild(deleteModal);

    // 添加事件监听
    const cancelButton = deleteModal.querySelector(".delete-cancel-button");
    const confirmButton = deleteModal.querySelector(".delete-confirm-button");

    const closeModal = () => {
      document.body.removeChild(deleteModal);
      document.removeEventListener("keydown", handleEsc);
    };

    cancelButton.addEventListener("click", closeModal);

    confirmButton.addEventListener("click", async () => {
      try {
        showLoading(true, translations[currentLanguage].saving_profile);

        await invoke("delete_profile", {
          profileId: globalState.currentProfile,
        });

        closeModal();
        showToast(translations[currentLanguage].profile_deleted, "success");

        // 增加更长延迟，确保删除操作完全完成，避免并发锁定冲突
        setTimeout(async () => {
          if (window.navigationPanel) {
            try {
              await window.navigationPanel.refreshProfileList();

              // 更长延迟确保后端锁定完全释放
              setTimeout(() => {
                // 清理已删除配置的状态
                globalState.hasUnsavedChanges = false;
                globalState.currentProfile = null;

                // 切换到第一个可用的配置文件
                if (globalState.profiles.length > 0) {
                  window.navigationPanel.switchToProfile(
                    globalState.profiles[0].id
                  );
                }

                // 最后刷新状态图标
                setTimeout(() => {
                  if (window.navigationPanel) {
                    window.navigationPanel.loadProfileStatuses();
                  }
                }, 300);
              }, 800); // 从200ms增加到800ms
            } catch (error) {
              console.error(
                "Failed to refresh profile list after deletion:",
                error
              );
              // 静默处理错误，不显示额外通知
            }
          }
        }, 500); // 从300ms增加到500ms
      } catch (error) {
        console.error("Failed to delete profile:", error);
        showToast(
          `${translations[currentLanguage].profile_delete_error}: ${error}`,
          "error"
        );
        closeModal();
      } finally {
        showLoading(false);
        // 按钮状态会在加载状态结束后自动更新
      }
    });

    // ESC键关闭
    const handleEsc = (e) => {
      if (e.key === "Escape") {
        closeModal();
      }
    };
    document.addEventListener("keydown", handleEsc);

    // 点击背景关闭
    deleteModal.addEventListener("click", (e) => {
      if (e.target === deleteModal) {
        closeModal();
      }
    });
  }

  // 自定义警告对话框，使用系统图标
  showCustomAlert(message) {
    // 创建自定义对话框
    const alertModal = document.createElement("div");
    alertModal.className = "modal-overlay";
    alertModal.style.display = "flex";

    alertModal.innerHTML = `
            <div class="modal-content alert-modal">
                <div class="modal-header">
                    <div class="alert-icon">⚠️</div>
                    <h3>${
                      translations[currentLanguage].validation_error ||
                      "Validation Error"
                    }</h3>
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
    const okButton = alertModal.querySelector(".alert-ok-button");
    okButton.addEventListener("click", () => {
      document.body.removeChild(alertModal);
    });

    // ESC键关闭
    const handleEsc = (e) => {
      if (e.key === "Escape") {
        document.body.removeChild(alertModal);
        document.removeEventListener("keydown", handleEsc);
      }
    };
    document.addEventListener("keydown", handleEsc);

    // 点击背景关闭
    alertModal.addEventListener("click", (e) => {
      if (e.target === alertModal) {
        document.body.removeChild(alertModal);
        document.removeEventListener("keydown", handleEsc);
      }
    });
  }
}

// Settings Manager Component
class SettingsManager {
  constructor() {
    this.ignoredFieldsList = document.getElementById("ignored-fields-list");
    this.addFieldButton = document.getElementById("add-field-button");
    this.fieldInputContainer = document.getElementById("field-input-container");
    this.newFieldInput = document.getElementById("new-field-input");
    this.confirmAddButton = document.getElementById("confirm-add-field");
    this.cancelAddButton = document.getElementById("cancel-add-field");
    this.resetFieldsButton = document.getElementById("reset-fields-button");
    this.saveSettingsButton = document.getElementById("save-settings-button");
    
    this.currentFields = [];
    this.isAddingField = false;
    
    this.initializeEventListeners();
  }
  
  initializeEventListeners() {
    // Add field button
    this.addFieldButton.addEventListener("click", () => {
      this.showAddFieldInput();
    });
    
    // Confirm add field
    this.confirmAddButton.addEventListener("click", () => {
      this.handleAddField();
    });
    
    // Cancel add field
    this.cancelAddButton.addEventListener("click", () => {
      this.hideAddFieldInput();
    });
    
    // Reset to default
    this.resetFieldsButton.addEventListener("click", () => {
      this.resetToDefault();
    });
    
    // Save settings
    this.saveSettingsButton.addEventListener("click", () => {
      this.saveSettings();
    });
    
    // Enter key in input field
    this.newFieldInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        this.handleAddField();
      } else if (e.key === "Escape") {
        this.hideAddFieldInput();
      }
    });
    
    // Input validation
    this.newFieldInput.addEventListener("input", () => {
      this.validateFieldInput();
    });
  }
  
  async loadSettings() {
    try {
      showLoading(true, "Loading settings...");
      
      // Load ignored fields
      this.currentFields = await invoke("get_ignored_fields");
      this.renderFieldsList();
      
      // Update profiles count
      updateProfilesCount();
      
      // Update Claude directory display
      const profilesInfo = await invoke("get_profiles_info");
      const claudeDirectoryElement = document.getElementById("claude-directory");
      if (claudeDirectoryElement) {
        claudeDirectoryElement.textContent = profilesInfo.claude_directory;
      }
      
    } catch (error) {
      console.error("Failed to load settings:", error);
      showToast(translations[currentLanguage].settings_save_error, "error");
    } finally {
      showLoading(false);
    }
  }
  
  renderFieldsList() {
    this.ignoredFieldsList.innerHTML = "";
    
    this.currentFields.forEach((field, index) => {
      const fieldItem = document.createElement("div");
      fieldItem.className = "field-item";
      
      fieldItem.innerHTML = `
        <span class="field-name">${this.escapeHtml(field)}</span>
        <button class="remove-field-button" data-field-index="${index}" title="Remove field">
          <span class="button-icon">✗</span>
        </button>
      `;
      
      // Add remove event listener
      const removeButton = fieldItem.querySelector(".remove-field-button");
      removeButton.addEventListener("click", () => {
        this.removeField(index);
      });
      
      this.ignoredFieldsList.appendChild(fieldItem);
    });
    
    // Show empty state if no fields
    if (this.currentFields.length === 0) {
      const emptyState = document.createElement("div");
      emptyState.className = "empty-state";
      emptyState.textContent = "No ignored fields configured";
      this.ignoredFieldsList.appendChild(emptyState);
    }
  }
  
  showAddFieldInput() {
    if (this.isAddingField) return;
    
    this.isAddingField = true;
    this.fieldInputContainer.style.display = "flex";
    this.addFieldButton.disabled = true;
    this.newFieldInput.value = "";
    this.newFieldInput.focus();
    this.validateFieldInput();
  }
  
  hideAddFieldInput() {
    this.isAddingField = false;
    this.fieldInputContainer.style.display = "none";
    this.addFieldButton.disabled = false;
    this.newFieldInput.value = "";
    this.newFieldInput.classList.remove("invalid");
  }
  
  validateFieldInput() {
    const fieldName = this.newFieldInput.value.trim();
    let isValid = true;
    let errorMessage = "";
    
    if (fieldName.length === 0) {
      isValid = false;
    } else if (fieldName.length > 100) {
      isValid = false;
      errorMessage = translations[currentLanguage].field_name_too_long;
    } else if (/[\s{}[\]"'\\]/.test(fieldName)) {
      isValid = false;
      errorMessage = translations[currentLanguage].field_name_invalid;
    } else if (this.currentFields.includes(fieldName)) {
      isValid = false;
      errorMessage = translations[currentLanguage].field_already_exists;
    }
    
    this.confirmAddButton.disabled = !isValid;
    this.newFieldInput.classList.toggle("invalid", fieldName.length > 0 && !isValid);
    this.newFieldInput.title = errorMessage;
  }
  
  async handleAddField() {
    const fieldName = this.newFieldInput.value.trim();
    
    if (!fieldName || this.confirmAddButton.disabled) {
      return;
    }
    
    try {
      // Add field to current list
      this.currentFields.push(fieldName);
      
      // Re-render list
      this.renderFieldsList();
      
      // Hide input
      this.hideAddFieldInput();
      
      showToast(`Field "${fieldName}" added`, "success");
      
    } catch (error) {
      console.error("Failed to add field:", error);
      showToast("Failed to add field", "error");
    }
  }
  
  removeField(index) {
    if (index >= 0 && index < this.currentFields.length) {
      const fieldName = this.currentFields[index];
      this.currentFields.splice(index, 1);
      this.renderFieldsList();
      showToast(`Field "${fieldName}" removed`, "success");
    }
  }
  
  async resetToDefault() {
    try {
      showLoading(true, "Resetting to default...");
      
      const defaultFields = await invoke("get_default_ignored_fields");
      this.currentFields = [...defaultFields];
      this.renderFieldsList();
      
      showToast("Reset to default fields", "success");
      
    } catch (error) {
      console.error("Failed to reset to default:", error);
      showToast("Failed to reset to default", "error");
    } finally {
      showLoading(false);
    }
  }
  
  async saveSettings() {
    try {
      showLoading(true, translations[currentLanguage].saving_settings);
      
      // Update ignored fields
      await invoke("update_ignored_fields", { 
        fields: this.currentFields 
      });
      
      showToast(translations[currentLanguage].settings_saved, "success");
      
      // Refresh profile statuses to reflect the new ignored fields
      setTimeout(() => {
        if (window.navigationPanel) {
          window.navigationPanel.loadProfileStatuses();
        }
      }, 300);
      
    } catch (error) {
      console.error("Failed to save settings:", error);
      showToast(translations[currentLanguage].settings_save_error, "error");
    } finally {
      showLoading(false);
    }
  }
  
  escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }
}

// Save As Modal Component
class SaveAsModal {
  constructor() {
    this.modal = document.getElementById("save-as-modal");
    this.profileNameInput = document.getElementById("profile-name-input");
    this.filenamePreview = document.getElementById("filename-preview");
    this.cancelButton = document.getElementById("modal-cancel-button");
    this.saveButton = document.getElementById("modal-save-button");
    this.closeButton = document.getElementById("modal-close-button");

    this.currentContent = "";

    this.initializeEventListeners();
  }

  initializeEventListeners() {
    // Profile name input
    this.profileNameInput.addEventListener("input", () => {
      this.updateFilenamePreview();
      this.validateInput();
    });

    // Modal buttons
    this.cancelButton.addEventListener("click", () => this.hide());
    this.closeButton.addEventListener("click", () => this.hide());
    this.saveButton.addEventListener("click", () => this.handleSave());

    // Close on overlay click
    this.modal.addEventListener("click", (e) => {
      if (e.target === this.modal) {
        this.hide();
      }
    });

    // ESC key
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && this.modal.style.display !== "none") {
        this.hide();
      }
    });
  }

  show(content) {
    this.currentContent = content;
    this.profileNameInput.value = "";
    this.updateFilenamePreview();
    this.validateInput();
    this.modal.style.display = "flex";
    this.profileNameInput.focus();
  }

  hide() {
    this.modal.style.display = "none";
    this.profileNameInput.value = "";
  }

  updateFilenamePreview() {
    const name = this.profileNameInput.value.trim();
    if (name) {
      this.filenamePreview.textContent = `${name}.settings.json`;
    } else {
      this.filenamePreview.textContent = "profile-name.settings.json";
    }
  }

  validateInput() {
    const name = this.profileNameInput.value.trim();
    
    // Enhanced validation to match backend rules
    let isValid = true;
    let errorMessage = "";
    
    if (name.length === 0) {
      isValid = false;
    } else if (name.length > 200) {
      isValid = false;
      errorMessage = "Profile name too long (max 200 characters)";
    } else if (/[\/\\:*?"<>|\0\t\r\n]/.test(name)) {
      isValid = false;
      errorMessage = "Contains invalid characters";
    } else if (name.startsWith(' ') || name.endsWith(' ') || name.startsWith('.') || name.endsWith('.')) {
      isValid = false;
      errorMessage = "Cannot start or end with spaces or dots";
    } else {
      // Check Windows reserved names
      const windowsReserved = ['CON', 'PRN', 'AUX', 'NUL', 'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9', 'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9'];
      const appReserved = ['current', 'settings', 'backup', 'temp', 'tmp'];
      
      if (windowsReserved.includes(name.toUpperCase()) || appReserved.includes(name.toLowerCase())) {
        isValid = false;
        errorMessage = "Reserved name cannot be used";
      } else if (name.includes('.') && name.split('.').some(part => part === '')) {
        isValid = false;
        errorMessage = "Cannot contain consecutive dots";
      }
    }

    this.saveButton.disabled = !isValid;

    // Update input style and show error message
    if (name.length > 0) {
      this.profileNameInput.classList.toggle("invalid", !isValid);
      if (!isValid && errorMessage) {
        this.profileNameInput.title = errorMessage;
      } else {
        this.profileNameInput.title = "";
      }
    } else {
      this.profileNameInput.classList.remove("invalid");
      this.profileNameInput.title = "";
    }
  }

  async handleSave() {
    const profileName = this.profileNameInput.value.trim();

    if (!profileName) {
      return;
    }

    try {
      showLoading(true, translations[currentLanguage].saving_profile);

      const filePath = await invoke("create_new_profile", {
        profileName,
        content: this.currentContent,
      });

      this.hide();
      showToast(translations[currentLanguage].profile_created, "success");

      // 延迟刷新以避免与通知冲突，并确保后端操作完成
      setTimeout(async () => {
        if (window.navigationPanel) {
          try {
            await window.navigationPanel.refreshProfileList();
            // 确保当前配置内容正确显示（另存为操作不应该改变用户当前查看的配置）
            if (window.contentEditor && globalState.currentProfile) {
              setTimeout(() => {
                window.contentEditor.loadProfile(globalState.currentProfile);
              }, 100);
            }
            // 再延迟刷新状态图标，确保列表已更新
            setTimeout(() => {
              window.navigationPanel.loadProfileStatuses();
            }, 200);
          } catch (error) {
            console.error(
              "Failed to refresh profile list after creation:",
              error
            );
            // 静默处理错误，不显示额外通知
          }
        }
      }, 300);
    } catch (error) {
      console.error("Failed to create profile:", error);
      showToast(
        `${translations[currentLanguage].profile_create_error}: ${error}`,
        "error"
      );
    } finally {
      showLoading(false);
      // 按钮状态会在加载状态结束后自动更新
    }
  }
}

function updateProfilesCount() {
  const profilesCountElement = document.getElementById("profiles-count");
  if (profilesCountElement && globalState.profiles) {
    profilesCountElement.textContent = globalState.profiles.length.toString();
  }
}

// Show tray popup menu (Windows workaround)
function showTrayPopupMenu(menuData) {
  console.log("Showing tray popup menu:", menuData);
  
  // Remove any existing tray menu
  const existingMenu = document.getElementById("tray-popup-menu");
  if (existingMenu) {
    existingMenu.remove();
  }
  
  // Create popup menu
  const menuContainer = document.createElement("div");
  menuContainer.id = "tray-popup-menu";
  menuContainer.className = "tray-popup-menu";
  
  // Position near cursor (approximate)
  menuContainer.style.cssText = `
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: white;
    border: 1px solid #ccc;
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 10000;
    min-width: 200px;
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 14px;
  `;
  
  // Add menu items
  menuData.items.forEach(item => {
    if (item.id === "separator") {
      const separator = document.createElement("div");
      separator.style.cssText = `
        height: 1px;
        background: #e0e0e0;
        margin: 4px 0;
      `;
      menuContainer.appendChild(separator);
    } else {
      const menuItem = document.createElement("div");
      menuItem.className = "tray-menu-item";
      menuItem.textContent = item.label;
      
      // Style menu item
      menuItem.style.cssText = `
        padding: 8px 16px;
        cursor: pointer;
        transition: background-color 0.1s;
        ${item.enabled === false ? 'color: #999; cursor: default;' : ''}
      `;
      
      // Add hover effect for enabled items
      if (item.enabled !== false) {
        menuItem.addEventListener('mouseenter', () => {
          menuItem.style.backgroundColor = '#f0f0f0';
        });
        
        menuItem.addEventListener('mouseleave', () => {
          menuItem.style.backgroundColor = 'transparent';
        });
        
        // Add click handler
        menuItem.addEventListener('click', () => {
          handleTrayMenuItemClick(item.id);
          menuContainer.remove();
        });
      }
      
      menuContainer.appendChild(menuItem);
    }
  });
  
  // Add to page
  document.body.appendChild(menuContainer);
  
  // Auto-remove after 5 seconds or on outside click
  const removeMenu = () => {
    if (menuContainer.parentNode) {
      menuContainer.remove();
    }
    document.removeEventListener('click', outsideClickHandler);
  };
  
  const outsideClickHandler = (e) => {
    if (!menuContainer.contains(e.target)) {
      removeMenu();
    }
  };
  
  setTimeout(() => {
    document.addEventListener('click', outsideClickHandler);
  }, 100);
  
  setTimeout(removeMenu, 5000);
}

// Handle tray menu item clicks
function handleTrayMenuItemClick(itemId) {
  console.log("Tray menu item clicked:", itemId);
  
  switch (itemId) {
    case "settings":
      // Show settings window (current window)
      console.log("Opening settings...");
      break;
      
    case "exit":
      // Exit application
      console.log("Exiting application...");
      if (invoke) {
        invoke("exit_application").catch(error => {
          console.error("Failed to exit application:", error);
        });
      }
      break;
      
    default:
      if (itemId.startsWith("profile_")) {
        const profileName = itemId.replace("profile_", "");
        console.log("Switching to profile:", profileName);
        // TODO: Implement profile switching via backend
      }
      break;
  }
}

// Initialize application
function initializeApp() {
  console.log("Initializing enhanced CCCS settings...");

  // Initialize Tauri APIs
  if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.window) {
    invoke = window.__TAURI__.core.invoke;
    getCurrentWindow = window.__TAURI__.window.getCurrentWindow;
    console.log("Tauri APIs initialized successfully");
  } else {
    console.error("Tauri APIs not available");
    return;
  }

  // Debug: 检查当前窗口尺寸
  console.log(
    "Current window size:",
    window.innerWidth,
    "x",
    window.innerHeight
  );
  console.log("Screen size:", window.screen.width, "x", window.screen.height);

  // Log current window size (let Rust code handle sizing)
  if (getCurrentWindow) {
    getCurrentWindow()
      .innerSize()
      .then((size) => {
        console.log("Current window size:", size.width, "x", size.height);
      })
      .catch((e) => console.log("Failed to get window size:", e));
  }

  // Detect and set language
  currentLanguage = detectSystemLanguage();
  updateTexts();

  // Initialize components
  window.navigationPanel = new NavigationPanel();
  window.contentEditor = new ContentEditor();
  window.saveAsModal = new SaveAsModal();
  window.settingsManager = new SettingsManager();

  // Debug: 检查CSS是否正确应用
  setTimeout(() => {
    const jsonEditor = document.getElementById("json-editor");
    const editorControls = document.querySelector(".editor-controls");

    if (jsonEditor) {
      const style = window.getComputedStyle(jsonEditor);
      console.log("JSON Editor computed style:");
      console.log("- flex:", style.flex);
      console.log("- height:", style.height);
      console.log("- padding-bottom:", style.paddingBottom);
    }

    if (editorControls) {
      const style = window.getComputedStyle(editorControls);
      console.log("Editor Controls computed style:");
      console.log("- position:", style.position);
      console.log("- bottom:", style.bottom);
      console.log("- right:", style.right);
    }
  }, 2000);

  // Load initial data
  window.navigationPanel.loadProfiles();

  // Set up language selector
  const languageSelect = document.getElementById("language-select");
  if (languageSelect) {
    languageSelect.addEventListener("change", (e) => {
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

  // Set up window control buttons
  const minimizeButton = document.getElementById("minimize-button");
  const exitButton = document.getElementById("exit-button");
  
  if (minimizeButton) {
    minimizeButton.addEventListener("click", async () => {
      try {
        // 关闭设置窗口，返回托盘运行状态
        if (getCurrentWindow) {
          await getCurrentWindow().close();
        }
      } catch (error) {
        console.error("Failed to close window:", error);
      }
    });
  }
  
  if (exitButton) {
    exitButton.addEventListener("click", async () => {
      try {
        // 退出整个应用程序
        await invoke("exit_application");
      } catch (error) {
        console.error("Failed to exit application:", error);
        // 如果后端命令失败，尝试直接关闭窗口
        if (getCurrentWindow) {
          await getCurrentWindow().close();
        }
      }
    });
  }

  // Set up tray popup menu listener for Windows
  console.log("Setting up tray popup menu listener...");
  console.log("Tauri APIs available:", window.__TAURI__);
  
  if (window.__TAURI__ && window.__TAURI__.event) {
    console.log("Tauri event API available:", window.__TAURI__.event);
    const { listen } = window.__TAURI__.event;
    
    console.log("Setting up event listener for 'show_tray_popup_menu'");
    
    // Listen for tray popup menu events from backend
    listen('show_tray_popup_menu', (event) => {
      console.log("🎯 Received tray popup menu event:", event);
      try {
        showTrayPopupMenu(event.payload);
      } catch (error) {
        console.error("Error showing tray popup menu:", error);
      }
    }).then(() => {
      console.log("✅ Tray popup menu listener set up successfully");
    }).catch(error => {
      console.error("❌ Failed to set up tray popup menu listener:", error);
    });
    
    // Also listen for test events to verify the system works
    listen('test_event', (event) => {
      console.log("🧪 Test event received:", event);
    }).then(() => {
      console.log("✅ Test event listener set up successfully");
    }).catch(error => {
      console.error("❌ Failed to set up test event listener:", error);
    });
    
  } else {
    console.error("❌ Tauri event API not available");
    console.log("Available Tauri APIs:", Object.keys(window.__TAURI__ || {}));
  }

  // Update profiles count in About section
  updateProfilesCount();

  console.log("Enhanced CCCS settings initialized successfully");
}

// Start the application when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initializeApp);
} else {
  initializeApp();
}
