# mdnote

命令行 Markdown 笔记管理工具，Rust 期末大作业。

## 能干嘛

- 建笔记、删笔记、改笔记
- 按标签/分类/日期筛选
- 搜内容（支持正则）
- 导出成 json/html/txt
- 看统计信息

## 编译运行

需要装好 Rust 和 Cargo，版本没啥特别要求，1.70 以上都行。

```bash
# 编译
cargo build --release

# 直接跑
cargo run -- <命令> [选项]

# 或者装到系统里，以后直接用 mdnote 命令
cargo install --path .
```

编译完的二进制在 `target/release/mdnote`。

## 怎么用

### 建笔记

```bash
mdnote new "我的学习笔记"
```

会在 notes/ 目录下生成一个 .md 文件，自带 YAML 头：

```
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
mdnote list                      # 全部
mdnote list --tag rust           # 按标签
mdnote list --category 学习      # 按分类
mdnote list --date 2026-05-31    # 按日期
mdnote list --date-start 2026-01-01 --date-end 2026-12-31  # 日期范围
```

### 搜内容

```bash
mdnote search "关键词"           # 普通搜索，不区分大小写
mdnote search "pattern" --regex  # 正则搜索
```

会匹配标题和正文，显示匹配的行号。

### 其他命令

```bash
mdnote show <id>                 # 看笔记详情，id 就是文件名去掉.md
mdnote edit <id>                 # 用编辑器打开（优先vscode，没有就用记事本）
mdnote delete <id>               # 删笔记
mdnote tag <id> rust             # 加标签
mdnote untag <id> rust           # 去标签
mdnote category <id> 学习        # 设分类
mdnote stats                     # 看统计
mdnote export --format json      # 导出所有笔记为json
mdnote export --format html      # 导出html
mdnote export --format text      # 导出纯文本
mdnote export --format json --output backup.json  # 导出到指定文件
```

默认笔记放在当前目录的 notes/ 下，想换目录用 `--dir`：

```bash
mdnote --dir /path/to/notes list
```

## 项目结构

```
src/
├── main.rs      # 入口
├── cli.rs       # 命令行解析和路由
├── model.rs     # 数据结构定义
├── storage.rs   # 文件读写、YAML解析
├── search.rs    # 搜索和过滤
└── export.rs    # 导出和统计
```

## 依赖

- clap 4.5 — 命令行参数解析
- serde / serde_json 1.0 — 序列化，json导出用的
- chrono 0.4 — 日期处理
- regex 1.10 — 正则搜索
- walkdir 2.5 — 遍历目录

## Rust 特性

作业要求体现的那些东西：

- **ownership / borrowing**：搜索和过滤那块大量用的引用，尽量不拷贝数据
- **struct / enum**：Note、Metadata、Tag 是结构体，Command、ExportFormat、StorageError 是枚举
- **trait**：Exportable trait 做导出接口，JsonExporter/HtmlExporter/TextExporter 分别实现
- **泛型**：filter_notes 接受 Fn(&Note) -> bool 闭包，不限定具体类型
- **Result**：所有可能炸的操作都返回 Result，用 ? 往上抛
- **模块化**：5个 mod 各管各的

## 测试

```bash
cargo test                        # 跑全部
cargo test --bin mdnote           # 只跑单元测试
cargo test --test integration_test  # 只跑集成测试
```

46 个单元测试 + 4 个集成测试，都能过。

## 代码规范

```bash
cargo fmt    # 格式化
cargo clippy # 静态检查，0 warning
```

## 许可证

MIT
