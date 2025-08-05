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
    saving_settings: "Saving settings...",

    // New translations for enhanced UI
    profiles_section_title: "Profiles",
    about_nav_item: "About",
    current_profile_title: "Current",
    json_editor_title: "Configuration Editor",
    json_editor_hint: "Edit your configuration in JSON format",
    json_editor_placeholder: "Loading configuration...",
    save_button: "Save",
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
    saving_settings: "正在保存设置...",

    // New translations for enhanced UI
    profiles_section_title: "配置文件",
    about_nav_item: "关于",
    current_profile_title: "当前配置",
    json_editor_title: "配置编辑器",
    json_editor_hint: "以JSON格式编辑您的配置",
    json_editor_placeholder: "正在加载配置...",
    save_button: "保存",
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
    this.aboutNavItem = document.querySelector(".about-nav-item");

    this.initializeEventListeners();
  }

  initializeEventListeners() {
    // About navigation click
    this.aboutNavItem.addEventListener("click", () => {
      this.handleItemClick("about");
    });
  }

  async loadProfiles(shouldAutoSelect = true) {
    try {
      showLoading(true, translations[currentLanguage].loading_profiles);

      const profiles = await invoke("get_profiles_list");
      globalState.profiles = profiles;

      this.renderProfileList(profiles);

      // 只在初次加载时自动选择第一个配置，刷新时保持当前配置
      if (shouldAutoSelect && profiles.length > 0) {
        // 延迟切换，避免与loadProfiles的loading状态冲突
        setTimeout(() => {
          this.switchToProfile(profiles[0].id);
        }, 0);
      } else if (!shouldAutoSelect && globalState.currentProfile) {
        // 刷新时保持当前选中的配置，更新导航状态
        this.updateActiveState(globalState.currentProfile);
      }

      // Update profiles count in About section
      updateProfilesCount();
    } catch (error) {
      console.error("Failed to load profiles:", error);
      // 不在这里显示错误通知，避免与其他操作的成功通知冲突
      // showToast("Failed to load profiles", "error");
    } finally {
      showLoading(false);
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
      text.textContent = profile.display_name;

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
    // 先更新全局状态
    globalState.currentProfile = profileId;
    globalState.hasUnsavedChanges = false;

    // 更新导航栏的激活状态
    this.updateActiveState(profileId);

    // 通知内容编辑器加载配置
    if (window.contentEditor) {
      window.contentEditor.loadProfile(profileId);
    }
  }

  updateActiveState(profileId) {
    // Remove active class from all items
    document.querySelectorAll(".nav-item").forEach((item) => {
      item.classList.remove("active");
    });

    // Add active class to selected item
    if (profileId === "about") {
      // Handle about navigation item
      const aboutItem = document.querySelector(".about-nav-item");
      if (aboutItem) {
        aboutItem.classList.add("active");
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
    this.aboutView = document.getElementById("about-view");
    this.saveButton = document.getElementById("save-button");
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
    try {
      showLoading(true, translations[currentLanguage].loading_content);

      if (profileId === "about") {
        this.showAboutContent();
        return;
      }

      this.showEditorContent();

      // Update title
      const profile = globalState.profiles.find((p) => p.id === profileId);
      if (profile) {
        this.profileTitle.textContent = profile.display_name;
      }

      // 显示或隐藏删除按钮（Current配置文件不显示删除按钮）
      if (this.deleteButton) {
        if (profileId === "current") {
          this.deleteButton.style.display = "none";
        } else {
          this.deleteButton.style.display = "inline-flex";
        }
      }

      // Load content
      const content = await invoke("load_profile_content", { profileId });
      this.jsonEditor.value = content;
      this.currentContent = content;
      this.originalContent = content;

      globalState.hasUnsavedChanges = false;
    } catch (error) {
      console.error("Failed to load profile content:", error);
      // 不显示错误通知，避免与其他操作的通知冲突
      // showToast("Failed to load profile content", "error");
      this.jsonEditor.value = "";
    } finally {
      showLoading(false);
      // 使用 setTimeout 确保状态更新在下一个事件循环中执行
      setTimeout(() => {
        this.updateSaveButtonState();
      }, 0);
    }
  }

  showEditorContent() {
    console.log("Showing editor content");
    this.jsonEditorView.style.display = "flex";
    this.aboutView.style.display = "none";

    // 按钮状态将在loadProfile完成后统一更新，避免竞争条件
  }

  showAboutContent() {
    console.log("Showing about content");
    this.jsonEditorView.style.display = "none";
    this.aboutView.style.display = "flex";
    this.profileTitle.textContent =
      translations[currentLanguage].about_nav_item;

    // About页面不需要更新编辑按钮状态，因为编辑按钮区域已经隐藏了
  }

  handleContentChange() {
    // 只有在有效配置文件时才处理内容变化
    if (!globalState.currentProfile || globalState.currentProfile === "about") {
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
      globalState.currentProfile && globalState.currentProfile !== "about";

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
    if (!globalState.currentProfile || globalState.currentProfile === "about") {
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
      globalState.currentProfile === "about"
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

  // Set window size programmatically
  if (getCurrentWindow) {
    // 先获取当前尺寸
    getCurrentWindow()
      .innerSize()
      .then((size) => {
        console.log("Tauri window size:", size.width, "x", size.height);

        // 尝试设置新尺寸
        return getCurrentWindow().setSize(
          size.type === "Physical"
            ? { type: "Physical", width: 1400, height: 1300 }
            : { type: "Logical", width: 1400, height: 1300 }
        );
      })
      .then(() => {
        console.log("Window resized to 1400x1300");
        // 验证是否成功
        return getCurrentWindow().innerSize();
      })
      .then((newSize) => {
        console.log("New window size:", newSize.width, "x", newSize.height);
      })
      .catch((e) => console.log("Window resize failed:", e));
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

  // Set up close button for About section
  const closeButton = document.getElementById("close-button");
  if (closeButton) {
    closeButton.addEventListener("click", async () => {
      try {
        await invoke("close_settings_window");
      } catch (error) {
        console.error("Failed to close window:", error);
      }
    });
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
