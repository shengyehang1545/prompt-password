# Prompt Password | 快捷密码

A lightweight local password quick-query tool for macOS and Windows. Instantly find and copy passwords via a global hotkey — fast, minimal, and distraction-free, just like an input method.

一个轻量的本地密码快捷查询工具，支持 macOS 和 Windows。通过全局快捷键即时查找并复制密码——快速、轻量、即用即走，体验如同输入法。

---

## Core Features | 核心功能

| Feature | Description | 功能 | 说明 |
|---------|-------------|------|------|
| Hotkey Summon | Global hotkey to bring up the search panel | 快捷键唤起 | 全局热键调出搜索面板 |
| Instant Search | Fuzzy search by site name / name / description / tags | 即时搜索 | 支持按网站名/名称/描述/标签进行模糊搜索 |
| Secure Display | Password shows first 2 chars, rest masked (e.g. `Ab******`) | 安全展示 | 密码仅显示前两位，后续打码（如 `Ab******`） |
| One-Click Copy | Auto-copy password to clipboard on selection | 一键复制 | 选中条目后自动复制密码到剪贴板 |

## Password Entry Data Model | 密码条目数据模型

Each password entry contains the following fields:

| Field | Description | 字段 | 说明 |
|-------|-------------|------|------|
| Name | Identifier for the password entry | 名称 | 密码条目标识名 |
| URL | Corresponding website URL | 网站链接 | 对应的网站 URL |
| Description | Supplementary notes | 描述 | 补充说明 |
| Alias | Hint when multiple accounts share the same password (e.g. "Birthday Password" → yyyymmdd format) | 密码别称 | 当多个账号共用同一密码时作提示用（如「生日密码」对应 yyyymmdd 格式） |
| Password | The actual password value | 密码 | 实际密码值 |
| Tags | Keywords for categorization and search | 标签 | 用于分类和搜索的关键词 |

## Target Platforms | 目标平台

- **macOS** (primary)
- **Windows**

## Project Structure (Planned) | 项目结构规划

```
prompt-password/
├── src/
│   ├── main/                  # Main process (Electron)
│   │   ├── index.ts           # Entry point
│   │   ├── hotkey.ts          # Global hotkey registration
│   │   └── clipboard.ts       # Clipboard operations
│   ├── renderer/              # UI layer
│   │   ├── index.html         # Search panel
│   │   ├── search.ts          # Search logic & fuzzy matching
│   │   └── mask.ts            # Password masking display
│   ├── store/                 # Data layer
│   │   ├── entries.ts         # Password entry CRUD
│   │   └── schema.ts          # Data model definition
│   └── crypto/                # Security layer
│       ├── encrypt.ts         # Encryption at rest
│       └── decrypt.ts         # Decryption for display
├── package.json
├── tsconfig.json
└── README.md
```

> **Note**: The structure above is a planning reference. Technology choices (e.g. Electron vs Tauri, storage format, encryption scheme) will be decided during Phase 1 implementation.

> **注**：以上结构为规划参考。技术选型（如 Electron vs Tauri、存储格式、加密方案）将在第一阶段实现时确定。

## Development Phases | 开发阶段

| Phase | Scope | 阶段 | 范围 |
|-------|-------|------|------|
| Phase 1 | Minimum viable — hotkey + search + copy | 第一阶段 | 最小可用 — 快捷键 + 搜索 + 复制 |
| Phase 2 | Core experience — complete happy path | 第二阶段 | 核心体验 — 完整主流程 |
| Phase 3 | Edge cases — error handling, polish | 第三阶段 | 边界处理 — 错误处理、打磨 |
| Phase 4 | Optimization — performance, monitoring | 第四阶段 | 优化 — 性能、监控 |

## License

Private — for personal use only.
