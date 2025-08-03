# CCCS - Claude Code Configuration Switcher

[English](./README.md) | 中文文档

一个轻量级桌面应用程序，用于快速切换 Claude Code 配置文件。

## 功能特性

- **快速配置切换**：通过系统托盘单击即可切换不同的 Claude Code 配置
- **智能状态指示器**：可视化显示配置文件状态：
  - ✅ **完全匹配** - 配置与当前设置完全一致
  - 🔄 **部分匹配** - 除 model 字段外完全一致（Claude Code 自动更新）
  - ❌ **错误** - 读取或解析配置文件失败
  - **无图标** - 配置与当前设置不同
- **自动检测**：自动检测 Claude Code 安装和配置文件
- **实时监控**：监控配置变化并相应更新状态
- **多语言支持**：支持中英文界面
- **系统托盘集成**：后台运行，资源占用最小

## 安装

### 系统要求

- 系统中必须已安装 Claude Code
- 支持 macOS、Windows 或 Linux

### 下载

目前请下载源代码自行编译，预构建的二进制文件将在未来版本中提供。

```bash
# 克隆仓库
git clone https://github.com/breakstring/cccs.git
cd cccs

# 安装依赖
npm install

# 生产环境构建
npm run tauri build
```

## 使用说明

### 系统托盘菜单

![托盘菜单](./images/traymenu.png)

*CCCS 系统托盘菜单显示不同的配置文件状态*

### 配置文件格式

CCCS 自动扫描 Claude Code 目录中的配置文件（macOS/Linux 为 `~/.claude/`，Windows 为 `%USERPROFILE%\.claude\`）。

**配置文件命名规范：**
- 配置文件必须遵循格式：`{配置名称}.settings.json`
- 示例：
  - `工作.settings.json`
  - `个人.settings.json`
  - `开发环境.settings.json`

**文件位置：**
- macOS/Linux：`~/.claude/`
- Windows：`%USERPROFILE%\.claude\`

**重要说明：**
- 主要的 `settings.json` 文件是当前激活的配置
- 配置文件应包含有效的 JSON 配置数据
- CCCS 在比较配置时会智能忽略 `model` 字段，因为 Claude Code 会自动更新此字段

### 快速开始

1. **启动 CCCS**：应用程序将出现在系统托盘中
2. **创建配置文件**：复制当前的 `~/.claude/settings.json` 创建配置文件（如 `工作.settings.json`）
3. **切换配置**：右键点击托盘图标并选择所需配置
4. **监控状态**：将鼠标悬停在托盘图标上以刷新配置状态

### 配置编辑器

![配置编辑器](./images/edit.png)

*CCCS 配置编辑器用于修改配置文件设置*

CCCS 包含内置的配置编辑器，允许您直接在应用程序内修改配置文件设置：

- **JSON 编辑器**：以 JSON 格式编辑配置文件，支持语法高亮
- **实时验证**：自动验证 JSON 语法和结构
- **配置文件管理**：创建、编辑和保存配置文件
- **安全编辑**：所有更改在应用前都会进行验证

### 设置

![设置页面](./images/settings.png)

*CCCS 设置页面及配置文件状态说明*

右键点击托盘图标并选择"设置"来访问配置：

- **语言设置**：在中英文之间切换
- **状态图标说明**：了解配置文件状态指示器的含义

## 关于此项目

本项目同时作为使用 Claude Code 进行 **Vibe Coding** 的示例展示。我们提供了原始提示词和使用 Kiro 的 SPECS 方法论开发过程中的产出，供参考：

- **原始提示词**：位于 `./doc/` - 包含初始项目需求和开发提示词
- **SPECS 开发过程**：位于 `./kiro/claude-config-switcher/` - 展示使用 Kiro 结构化方法的完整开发工作流程

如果您对 AI 辅助开发工作流程感兴趣，或想了解这个项目如何使用 Claude Code 从零开始构建，欢迎探索这些资源。

## 开发

### 开发环境要求

- Node.js（v16 或更高版本）
- Rust（最新稳定版）
- Tauri CLI

**注意**：本项目目前仅在 macOS 上进行了测试，因为开发者手头只有一台 Mac 笔记本。虽然 Tauri 框架理论上支持 Linux 和 Windows，但这些平台的用户可以自行摸索尝试兼容性。

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/breakstring/cccs.git
cd cccs

# 安装依赖
npm install

# 开发模式运行
npm run tauri dev

# 生产环境构建
npm run tauri build
```

### 项目结构

```
cccs/
├── src/                # 前端（TypeScript/Vite）
├── src-tauri/          # 后端（Rust/Tauri）
├── public/             # 静态资源
└── dist/               # 构建后的前端资源
```

## 贡献

1. Fork 本仓库
2. 创建功能分支
3. 提交你的更改
4. 推送到分支
5. 创建 Pull Request

## 许可证

本项目基于 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 开发路线图

- [x] 配置文件修改器
- [x] 修改默认图标
- [ ] 配置文件的 JSON Schema 校验

## 支持

如果您遇到任何问题或有疑问：

1. 查看 [Issues](https://github.com/breakstring/cccs/issues) 页面
2. 如果您的问题尚未报告，请创建新的 issue
3. 提供关于您的系统和问题的详细信息

---

由 [KZAI Lab](https://github.com/breakstring) 用 ❤️ 制作