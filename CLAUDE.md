# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Tauri application with a TypeScript/Vite frontend. The project combines a Rust backend (Tauri) with a web frontend built using Vite and TypeScript.

**CCCS (Claude Code Configuration Switcher)** is a system tray application that allows users to quickly switch between different Claude Code configuration profiles.

## Architecture

### Frontend (src/)
- **Entry point**: `src/main.ts` - Sets up the basic HTML structure and initializes the counter component
- **Components**: `src/counter.ts` - Simple counter functionality with click handlers
- **Styling**: `src/style.css` - Application styles
- **Build tool**: Vite with TypeScript support

### Backend (src-tauri/)
- **Entry point**: `src-tauri/src/main.rs` - Launches the Tauri application
- **Core logic**: `src-tauri/src/lib.rs` - Contains the main Tauri application setup with logging configuration
- **Build configuration**: `src-tauri/Cargo.toml` - Rust dependencies and build settings
- **Tauri configuration**: `src-tauri/tauri.conf.json` - App window settings, build commands, and bundle configuration

## Tauri Backend Architecture

### Core Modules
- **`app.rs`** - Main application state and initialization
- **`claude_detector.rs`** - Detects Claude Code installation and configuration files
- **`config_service.rs`** - Handles profile management and configuration file operations
- **`tray_service.rs`** - System tray icon and menu management
- **`monitor_service.rs`** - File monitoring for configuration changes
- **`settings_service.rs`** - Application settings management
- **`i18n_service.rs`** - Internationalization support (Chinese/English)
- **`validation.rs`** - JSON configuration validation framework
- **`types.rs`** - Common data structures and types
- **`error.rs`** - Error handling definitions

### Tauri Commands (API)
Available commands that can be called from frontend JavaScript:

- **`get_profiles_info()`** - Get summary information about profiles
- **`get_profiles_list()`** - Get detailed list of all profiles
- **`load_profile_content(profile_id: String)`** - Load content of a specific profile
- **`save_profile(profile_id: String, content: String)`** - Save changes to a profile
- **`create_new_profile(profile_name: String, content: String)`** - Create a new profile
- **`validate_json_content(content: String)`** - Validate JSON configuration
- **`close_settings_window()`** - Close the settings window

### Permissions Configuration
**Important**: `src-tauri/capabilities/default.json` defines Tauri permissions:
- Window operations: close, minimize, maximize, set-size, inner-size
- File system access for configuration files
- Dialog access for save/load operations

## Development Commands

### Frontend Development
- `npm run dev` - Start development server (runs Vite dev server on localhost:5173)
- `npm run build` - Build frontend for production (TypeScript compilation + Vite build)
- `npm run preview` - Preview production build

### Tauri Development
- `npm run tauri:dev` or `cargo tauri dev` - Run full application in development mode
- `npm run tauri:build` or `cargo tauri build` - Build production application
- Development is handled through the Tauri configuration in `tauri.conf.json`
- Frontend dev server runs on `http://localhost:5173`
- Build process: `npm run build` creates the `dist` directory for Tauri

### VS Code Development
**Launch configurations available (F5)**:
- **"Launch CCCS App"** - Run the full Tauri application
- **"Debug Frontend"** - Run only the Vite dev server
- **"Launch Full Stack"** - Run both frontend and backend simultaneously
- **"Attach to Chrome"** - Attach debugger to Chrome for frontend debugging

## Key Configuration Files

- `package.json` - Frontend dependencies and npm scripts
- `tsconfig.json` - TypeScript configuration with strict settings
- `src-tauri/tauri.conf.json` - Tauri app configuration including window settings and build commands
- `src-tauri/Cargo.toml` - Rust dependencies and library configuration
- `src-tauri/capabilities/default.json` - Tauri permissions configuration

## Settings Page File Structure (UPDATED - 2025-08-03)

**IMPORTANT**: 项目已成功重构为标准Tauri+Vite结构！

### 标准Tauri+Vite项目结构 (CURRENT)：
```
project-root/
├── index.html          ← Vite入口点 (在根目录)
├── package.json        ← 前端依赖
├── vite.config.js      ← Vite配置
├── src/                ← 前端源代码目录
│   ├── main.js        ← 前端入口JavaScript (ES6模块)
│   ├── style.css      ← 样式文件
│   └── ...            ← 其他前端源码
├── public/             ← 静态资源目录 (构建时复制到输出根目录)
│   ├── vite.svg       ← 静态资源
│   └── ...            ← 其他静态文件
├── src-tauri/          ← Tauri后端目录
│   ├── src/           ← Rust源代码
│   ├── Cargo.toml     ← Rust配置
│   └── tauri.conf.json ← Tauri配置
└── dist/               ← 构建输出目录 (生成)
    ├── index.html     ← 构建后的HTML
    ├── assets/        ← 打包后的JS/CSS
    └── ...            ← public/目录的静态文件
```

### 重构完成的改进：
1. ✅ **符合Vite最佳实践** - 标准的Vite项目结构
2. ✅ **ES6模块化** - 使用import/export语法
3. ✅ **清晰的职责分离** - 源码与静态资源明确分离
4. ✅ **简化的构建流程** - 标准npm scripts
5. ✅ **去除冗余配置** - 移除TypeScript依赖

### 当前的正确工作流程：
- **开发模式**: `npm run dev` (启动Vite开发服务器)
- **Tauri开发**: `npm run tauri:dev` (启动完整应用)
- **生产构建**: `npm run build` (构建前端)
- **Tauri打包**: `npm run tauri:build` (打包应用)

### 文件路径机制 (CURRENT)：
- **开发模式**: Vite serves static files from `public/` and sources from `src/`
  - `src/main.js` → `http://localhost:5173/src/main.js` (ES6模块)
  - `src/style.css` → 通过import自动加载
  - `public/vite.svg` → `http://localhost:5173/vite.svg`

- **生产模式**: Files are bundled into `dist/` directory
  - `src/main.js` + `src/style.css` → `dist/assets/index-[hash].js`
  - `public/vite.svg` → `dist/vite.svg`

### ALWAYS Edit These Files (Updated Structure)：
- `index.html` - HTML结构和布局
- `src/style.css` - 所有样式修改
- `src/main.js` - JavaScript功能和逻辑

### 构建和测试：
- **开发测试**: `npm run dev` 确保开发服务器正常
- **构建测试**: `npm run build` 确保生产构建成功
- **应用测试**: `npm run tauri:dev` 测试完整应用

### 备份文件位置：
- 原始文件备份在 `public_backup_[timestamp]/` 和 `index_backup_[timestamp].html`
- 如需回滚可参考备份文件

## Application Features

### System Tray Integration
- Always runs in system tray
- Menu shows all available profiles with status indicators
- Quick profile switching directly from tray menu
- Settings window accessible from tray menu

### Profile Management
- Automatically detects Claude Code directory (`~/.claude`)
- Scans for profile files (`*.settings.json`)
- Shows profile status: ✅ Full match, 🔄 Partial match, ❌ Error
- Create, edit, and save profiles through GUI

### Settings Interface
- **Left Panel**: Navigation between profiles and About section
- **Right Panel**: JSON editor with syntax highlighting and validation
- **Internationalization**: Chinese and English language support
- **Responsive Design**: Works on different screen sizes

### File Monitoring
- Automatically monitors configuration file changes
- Updates tray menu when files are modified externally
- Real-time status updates

## Testing and Debugging

### Debug Mode Features
- Extensive logging throughout the application
- Performance testing module (`performance_tests.rs`) available in debug builds
- Console debugging in settings window (F12)

### Common Issues
1. **Permission Errors**: Check `src-tauri/capabilities/default.json` for required permissions
2. **File Path Issues**: Ensure using absolute paths (`/settings.css`) not relative (`settings.css`)
3. **Build Issues**: Run `npm run build` after changes to `public/` directory
4. **Tauri API Not Available**: Check browser console for Tauri initialization errors

## Project Structure Notes

- Frontend assets are served from the `public/` directory
- Tauri icons are stored in `src-tauri/icons/` with multiple formats for different platforms
- The app uses a hybrid architecture where the frontend is built with Vite and bundled into the Tauri application
- No test framework is currently configured
- Application logs are available through Tauri's logging system