# mdnote — Markdown 笔记管理工具

一个用 Rust 编写的命令行 Markdown 笔记管理工具，支持笔记的增删改查、标签管理、全文搜索和多格式导出。

## 功能特性

- 📝 新建、编辑、删除笔记
- 📋 按标签/分类/日期筛选笔记
- 🔍 关键词全文搜索（不区分大小写）
- 🏷️ 标签管理（添加/移除）
- 📤 多格式导出（JSON / HTML / 纯文本）
- 📊 笔记统计信息
- 💾 自动 YAML front matter 管理

## 编译与运行

### 环境要求

- Rust 1.70+（推荐最新稳定版）
- Cargo（随 Rust 一起安装）

### 编译

```bash
cargo build --release
```

编译后的二进制文件在 `target/release/mdnote`。

### 直接运行

```bash
cargo run -- <命令> [选项]
```

### 安装到系统

```bash
cargo install --path .
```

安装后可以直接用 `mdnote` 命令。

## 使用方法

### 新建笔记

```bash
mdnote new "我的学习笔记"
```

会在 `notes/` 目录下创建 `my-learning-note.md` 文件，自动生成 YAML front matter：

```markdown
---
title: "我的学习笔记"
created: 2026-05-31
modified: 2026-05-31
tags: []
category: null
---
```

### 列出笔记

```bash
# 列出所有笔记
mdnote list

# 按标签筛选
mdnote list --tag rust

# 按分类筛选
mdnote list --category 学习

# 按日期筛选
mdnote list --date 2026-05-31
```

### 搜索笔记

```bash
mdnote search "关键词"
```

搜索会同时匹配标题和正文内容，不区分大小写，并显示匹配的行号。

### 查看笔记

```bash
mdnote show <笔记ID>
```

笔记 ID 就是文件名去掉 `.md` 后缀。

### 编辑笔记

```bash
mdnote edit <笔记ID>
```

会自动用 VSCode（code）或记事本打开笔记文件，编辑完成后自动更新修改日期。

### 删除笔记

```bash
mdnote delete <笔记ID>
```

### 标签管理

```bash
# 添加标签
mdnote tag <笔记ID> rust

# 移除标签
mdnote untag <笔记ID> rust
```

### 导出笔记

```bash
# 导出为 JSON 格式
mdnote export --format json

# 导出为 HTML
mdnote export --format html

# 导出为纯文本
mdnote export --format text

# 导出到文件
mdnote export --format json --output backup.json
```

### 统计信息

```bash
mdnote stats
```

### 指定笔记目录

默认使用当前目录下的 `notes/` 文件夹，可以通过 `--dir` 参数指定：

```bash
mdnote --dir /path/to/notes list
```

## 项目结构

```
src/
├── main.rs      # 程序入口
├── cli.rs       # 命令行参数解析与命令路由
├── model.rs     # 数据模型（Note, Tag, Metadata, Command, ExportFormat）
├── storage.rs   # 文件系统读写、YAML front matter 解析
├── search.rs    # 全文搜索、标签过滤、日期范围筛选
└── export.rs    # Exportable trait + 多格式导出 + 统计信息
```

## 依赖说明

| 依赖 | 版本 | 用途 |
|------|------|------|
| clap | 4.5 | 命令行参数解析 |
| serde / serde_json | 1.0 | 序列化/反序列化（JSON 导出） |
| chrono | 0.4 | 日期时间处理 |
| regex | 1.10 | 正则表达式搜索 |
| walkdir | 2.5 | 目录遍历 |

## Rust 特性体现

本项目充分体现了 Rust 的核心特性：

- **ownership / borrowing**：笔记的加载和搜索大量使用引用传递，避免数据拷贝
- **struct / enum**：`Note`、`Metadata`、`Tag` 等结构体；`Command`、`ExportFormat`、`StorageError` 等枚举
- **trait**：`Exportable` trait 定义导出接口，`JsonExporter`/`HtmlExporter`/`TextExporter` 分别实现
- **泛型**：`filter_notes` 函数接受任意 `Fn(&Note) -> bool` 闭包
- **Result 错误处理**：所有可能失败的操作都返回 `Result`，用 `?` 传播错误
- **模块化设计**：5 个模块各司其职，通过 `mod` 组织

## 测试

```bash
# 运行所有测试
cargo test

# 只运行单元测试
cargo test --bin mdnote

# 运行集成测试
cargo test --test integration_test
```

## 代码规范

```bash
# 格式化代码
cargo fmt

# 静态检查
cargo clippy
```

## 许可证

MIT
