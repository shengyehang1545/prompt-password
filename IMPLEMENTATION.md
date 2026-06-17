# Prompt Password 实施状态

> 更新时间：2026-06-17  
> 当前分支：`feat/cross-platform-initialization`  
> 当前目标：完成 macOS 密码快捷查询工具的首轮可用版本，并保留后续 Windows 适配路径。

## 当前结论

项目已从早期方案阶段进入可体验阶段。当前实现基于 Tauri 2、Rust、Preact 和 SQLite，核心路径是：

1. 用户输入 Master Password 解锁本地保险库。
2. 应用在 macOS 后台常驻，通过 `Cmd+Shift+K` 唤起搜索面板。
3. 用户搜索密码条目，点击或按 `Enter` 复制密码。
4. 密码内容加密保存，搜索字段明文保存以支持本地快速搜索。

Windows 不再单独重写一个完整版本，而是在同一套 Tauri/Rust/Preact 代码里保留跨平台结构。当前 Windows 托盘和开机启动行为仍需后续在真实 Windows 环境验收。

## 已完成范围

### 应用基础

- Tauri 2 项目结构已建立。
- 前端使用 Preact + TypeScript。
- 后端使用 Rust 命令暴露给前端。
- 本地数据使用 SQLite。
- macOS app 图标已重新生成，处理了 Dock 图标黑边问题。

### 保险库和安全

- 支持 Master Password 初始化和解锁。
- 使用 Argon2id 从 Master Password 派生密钥。
- 使用 AES-256-GCM 加密密码字段。
- 密钥只保存在当前进程内存里。
- 支持锁定保险库，锁定后需要重新输入 Master Password。
- 搜索结果不展示明文密码。
- 编辑已有条目时不会回显旧密码；密码输入留空表示保留原密码。

### 密码条目

- 支持新增、编辑、删除密码条目。
- 支持名称、URL、描述、别名、账号、标签等字段。
- 支持按本地字段搜索。
- 点击结果或键盘确认后复制密码。
- 复制完成后保留短暂状态反馈，避免因为鼠标点击导致体验中断。

### 密码模板

- 支持新增、编辑、删除模板。
- 模板密码输入支持可见/隐藏切换。
- 模板只负责填入当前表单密码。
- 模板与具体密码条目之间没有软连接。
- 编辑或删除模板不会影响任何已有密码条目。

### 密码生成器

- 默认生成 12 位密码。
- 至少包含 1 个数字、1 个大写字母、1 个小写字母。
- 默认包含 2 个特殊字符。
- 特殊字符只从 `!@#$%^&*,?` 中选择。
- 新增或编辑条目时可以直接生成密码并填入表单。

### macOS 后台运行

- macOS 下使用菜单栏托盘。
- 应用可隐藏 Dock 图标并作为后台工具运行。
- 托盘菜单支持显示窗口、锁定保险库、退出。
- 窗口隐藏后可通过全局快捷键重新唤起。

### 设置页面

- 支持开机启动设置。
- 支持自定义全局快捷键。
- 支持锁定保险库。
- 支持导入和导出。
- 默认快捷键：
  - macOS：`Cmd+Shift+K`
  - Windows 兼容配置：`Ctrl+Shift+K`

### 导入和导出

- 支持导出加密 JSON 备份。
- 支持导出明文 CSV。
- 支持导出明文 TXT。
- 导出前需要重新输入 Master Password 验证。
- JSON 导出不包含明文密码，适合备份和同步流转。
- CSV/TXT 是明文导出，只适合临时迁移或人工检查。
- CSV 对所有字段加双引号，并转义字段中的双引号；密码包含逗号、换行、引号时不会破坏 CSV 结构。
- 支持导入加密 JSON 备份。
- 导入时遇到相同 `id` 或名称的条目、模板会跳过，不覆盖已有数据。

### 输入体验

- 已关闭输入框的拼写检查、自动纠正和自动大小写。
- 搜索、解锁、添加、编辑、设置等输入路径都应用了同一套输入属性。

## 当前架构

```text
prompt-password/
├── src/
│   ├── App.tsx
│   ├── components/
│   │   ├── AddEntryForm.tsx
│   │   ├── EntryItem.tsx
│   │   ├── EntryList.tsx
│   │   ├── SearchInput.tsx
│   │   ├── SettingsModal.tsx
│   │   └── UnlockScreen.tsx
│   └── lib/
│       ├── inputGuards.ts
│       └── tauri.ts
└── src-tauri/
    └── src/
        ├── commands/
        │   ├── backup.rs
        │   ├── clipboard.rs
        │   ├── entry.rs
        │   ├── generator.rs
        │   ├── search.rs
        │   ├── settings.rs
        │   ├── template.rs
        │   └── vault.rs
        ├── core/
        │   ├── entry_input.rs
        │   └── vault.rs
        ├── crypto/
        ├── db/
        │   ├── init.rs
        │   └── queries.rs
        └── platform/
            ├── hotkey.rs
            ├── startup.rs
            ├── tray.rs
            └── window.rs
```

## 后端命令范围

| 命令 | 当前用途 |
|------|----------|
| `unlock_vault` | 初始化或解锁保险库 |
| `lock_vault` | 清空当前内存密钥并锁定保险库 |
| `search_entries` | 搜索密码条目 |
| `create_entry` | 新增条目 |
| `update_entry` | 编辑条目，密码为空时保留旧密码 |
| `delete_entry` | 删除条目 |
| `copy_entry_password` | 解密并复制指定条目的密码 |
| `generate_password` | 生成默认规则密码 |
| `list_password_templates` | 列出模板 |
| `create_password_template` | 新增模板 |
| `update_password_template` | 编辑模板 |
| `delete_password_template` | 删除模板 |
| `use_password_template` | 解密模板密码并填入表单 |
| `export_backup` | 导出 JSON/CSV/TXT |
| `import_backup` | 导入加密 JSON |
| `get_app_settings` | 读取设置 |
| `set_auto_start_enabled` | 设置开机启动 |
| `set_global_hotkey` | 设置全局快捷键 |

## 数据安全边界

- 密码字段加密保存。
- 搜索字段为明文，包括名称、URL、描述、别名、账号、标签。
- JSON 备份仍然是加密数据。
- CSV/TXT 导出是明文数据，导出前必须重新验证 Master Password。
- 当前开发和验收过程中不应读取本地数据库的密码明文字段，也不应通过查询绕过应用流程查看真实密码。

## 验收命令

```bash
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build -- --bundles app
```

构建后的 macOS app：

```text
src-tauri/target/release/bundle/macos/prompt-password.app
```

## 手工验收重点

1. 首次启动后设置 Master Password。
2. 使用 `Cmd+Shift+K` 唤起和隐藏窗口。
3. 新增、搜索、复制密码。
4. 新增密码时使用生成器，确认规则为 12 位且包含 2 个指定特殊字符。
5. 编辑条目时确认旧密码不会回显，留空不会替换旧密码。
6. 删除条目后确认搜索结果更新。
7. 新增、编辑、删除模板，确认不影响已有条目。
8. 设置页修改快捷键后确认新快捷键生效。
9. 开启开机启动后确认 LaunchAgent 创建成功。
10. 锁定保险库后确认需要重新输入 Master Password。
11. 导出 JSON，确认不能看到明文密码。
12. 导出 CSV/TXT 前确认需要重新输入 Master Password。
13. 导入 JSON 后确认重复条目不会覆盖已有数据。

## 未完成和后续事项

- Windows 托盘、窗口行为、全局快捷键需要在 Windows 真机或虚拟机中验收。
- Windows 开机启动暂未实现为完整设置项。
- 剪贴板自动清理本轮明确不做。
- 目前只支持导入加密 JSON，CSV/TXT 仅导出。
- 后续如果要做多端同步，应优先设计端到端加密的变更日志同步，不建议直接同步 SQLite 数据库文件。
