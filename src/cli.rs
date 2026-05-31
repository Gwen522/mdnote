// 命令行解析和执行逻辑

use crate::export::{compute_stats, create_exporter};
use crate::model::{Command, ExportFormat};
use crate::search::{self, Filter};
use crate::storage::Storage;
use std::path::PathBuf;

// ---- CLI 参数定义 ----

#[derive(clap::Parser, Debug)]
#[command(
    name = "mdnote",
    version,
    about = "一个 Markdown 笔记管理命令行工具",
    after_help = "示例:\n  mdnote new \"我的笔记\"\n  mdnote list --tag rust\n  mdnote list --date-start 2026-01-01 --date-end 2026-12-31\n  mdnote search \"关键词\"\n  mdnote search \"\\d+\" --regex\n  mdnote category my-note 学习\n  mdnote export --format json -o backup.json"
)]
pub struct Cli {
    #[arg(short, long, default_value = "notes", global = true)]
    pub dir: String,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// 新建笔记
    New { title: String },
    /// 列出笔记
    List {
        #[arg(short, long)]
        tag: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        date_start: Option<String>,
        #[arg(long)]
        date_end: Option<String>,
    },
    /// 搜索内容
    Search {
        keyword: String,
        #[arg(short, long)]
        regex: bool,
    },
    /// 查看笔记
    Show { id: String },
    /// 编辑笔记
    Edit { id: String },
    /// 删除笔记
    Delete { id: String },
    /// 添加标签
    Tag { id: String, tag: String },
    /// 移除标签
    Untag { id: String, tag: String },
    /// 设置分类
    Category { id: String, category: String },
    /// 导出笔记
    Export {
        #[arg(short, long, default_value = "json")]
        format: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 统计信息
    Stats,
}

// ---- 命令转换和执行 ----

pub fn parse_command(cli: &Cli) -> Option<Command> {
    match &cli.command {
        Some(Commands::New { title }) => Some(Command::New {
            title: title.clone(),
        }),
        Some(Commands::List {
            tag,
            category,
            date,
            date_start,
            date_end,
        }) => Some(Command::List {
            tag: tag.clone(),
            category: category.clone(),
            date: date.clone(),
            date_start: date_start.clone(),
            date_end: date_end.clone(),
        }),
        Some(Commands::Search { keyword, regex }) => Some(Command::Search {
            keyword: keyword.clone(),
            regex: *regex,
        }),
        Some(Commands::Show { id }) => Some(Command::Show { id: id.clone() }),
        Some(Commands::Edit { id }) => Some(Command::Edit { id: id.clone() }),
        Some(Commands::Delete { id }) => Some(Command::Delete { id: id.clone() }),
        Some(Commands::Tag { id, tag }) => Some(Command::Tag {
            id: id.clone(),
            tag: tag.clone(),
        }),
        Some(Commands::Untag { id, tag }) => Some(Command::Untag {
            id: id.clone(),
            tag: tag.clone(),
        }),
        Some(Commands::Category { id, category }) => Some(Command::Category {
            id: id.clone(),
            category: category.clone(),
        }),
        Some(Commands::Export { format, output }) => {
            let fmt = ExportFormat::from_str_opt(format).unwrap_or(ExportFormat::Json);
            Some(Command::Export {
                format: fmt,
                output: output.clone(),
            })
        }
        Some(Commands::Stats) => Some(Command::Stats),
        None => None,
    }
}

pub fn execute(cmd: &Command, storage: &Storage) -> Result<(), String> {
    match cmd {
        Command::New { title } => {
            let note = storage
                .create_note(title)
                .map_err(|e| format!("创建笔记失败: {}", e))?;
            println!("✅ 创建成功!");
            println!("{}", note);
            Ok(())
        }

        Command::List {
            tag,
            category,
            date,
            date_start,
            date_end,
        } => {
            let notes = storage
                .list_notes()
                .map_err(|e| format!("读取笔记列表失败: {}", e))?;

            let mut filters: Vec<Filter> = Vec::new();

            if let Some(t) = tag {
                filters.push(Filter::ByTag(t.clone()));
            }
            if let Some(c) = category {
                filters.push(Filter::ByCategory(c.clone()));
            }
            if let Some(d) = date {
                let target = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .map_err(|_| "日期格式不对，请用 YYYY-MM-DD".to_string())?;
                filters.push(Filter::ByDateRange(target, target));
            }
            if let (Some(ds), Some(de)) = (date_start, date_end) {
                let (start, end) =
                    search::parse_date_range(ds, de).map_err(|e| format!("{}", e))?;
                filters.push(Filter::ByDateRange(start, end));
            }

            let filtered: Vec<&crate::model::Note> = if filters.is_empty() {
                notes.iter().collect()
            } else {
                let combined = Filter::All(filters);
                search::filter_by_filter(&notes, &combined)
            };

            if filtered.is_empty() {
                println!("📭 没有找到匹配的笔记");
            } else {
                println!("📋 共 {} 条笔记:", filtered.len());
                for note in &filtered {
                    let tags_str = if note.metadata.tags.is_empty() {
                        String::new()
                    } else {
                        format!(
                            " [{}]",
                            note.tag_names()
                                .iter()
                                .map(|t| format!("#{}", t))
                                .collect::<Vec<_>>()
                                .join(" ")
                        )
                    };
                    let cat = note.metadata.category.as_deref().unwrap_or("");
                    println!(
                        "  {} - {}{}{}",
                        note.id,
                        note.metadata.title,
                        if cat.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", cat)
                        },
                        tags_str
                    );
                }
            }
            Ok(())
        }

        Command::Search { keyword, regex } => {
            let notes = storage
                .list_notes()
                .map_err(|e| format!("读取笔记列表失败: {}", e))?;

            if *regex {
                let results = search::search_regex(&notes, keyword)
                    .map_err(|e| format!("搜索失败: {}", e))?;
                if results.is_empty() {
                    println!("🔍 没有匹配正则 '{}' 的笔记", keyword);
                } else {
                    println!("🔍 找到 {} 条匹配的笔记:", results.len());
                    for result in results {
                        println!("\n  📄 {} ({})", result.note.metadata.title, result.note.id);
                        for (line_no, line) in &result.matched_lines {
                            println!("     L{}: {}", line_no, line);
                        }
                    }
                }
            } else {
                let results = search::search_keyword(&notes, keyword);
                if results.is_empty() {
                    println!("🔍 没有找到包含 '{}' 的笔记", keyword);
                } else {
                    println!("🔍 找到 {} 条匹配的笔记:", results.len());
                    for result in results {
                        println!("\n  📄 {} ({})", result.note.metadata.title, result.note.id);
                        for (line_no, line) in &result.matched_lines {
                            println!("     L{}: {}", line_no, line);
                        }
                    }
                }
            }
            Ok(())
        }

        Command::Show { id } => {
            let note = storage
                .load_note(id)
                .map_err(|e| format!("查看笔记失败: {}", e))?;
            println!("{}", note);
            let names = note.tag_names();
            if !names.is_empty() {
                println!("  标签名: {}", names.join(", "));
            }
            Ok(())
        }

        Command::Edit { id } => {
            if !storage.exists(id) {
                return Err(format!("笔记 '{}' 不存在", id));
            }
            let path = storage.notes_dir().join(format!("{}.md", id));
            open_editor(&path)?;
            // 编辑完更新修改时间
            if let Ok(mut note) = storage.load_note(id) {
                note.metadata.touch();
                let _ = storage.save_note(&note);
                println!("📝 笔记已更新");
            }
            Ok(())
        }

        Command::Delete { id } => {
            storage
                .delete_note(id)
                .map_err(|e| format!("删除笔记失败: {}", e))?;
            println!("🗑️  笔记 '{}' 已删除", id);
            Ok(())
        }

        Command::Tag { id, tag } => {
            let mut note = storage
                .load_note(id)
                .map_err(|e| format!("加载笔记失败: {}", e))?;
            note.add_tag(tag);
            storage
                .save_note(&note)
                .map_err(|e| format!("保存笔记失败: {}", e))?;
            println!("🏷️  已添加标签 #{}", tag);
            Ok(())
        }

        Command::Untag { id, tag } => {
            let mut note = storage
                .load_note(id)
                .map_err(|e| format!("加载笔记失败: {}", e))?;
            if !note.has_tag(tag) {
                return Err(format!("笔记上没有标签 '{}'", tag));
            }
            note.remove_tag(tag);
            storage
                .save_note(&note)
                .map_err(|e| format!("保存笔记失败: {}", e))?;
            println!("🏷️  已移除标签 #{}", tag);
            Ok(())
        }

        Command::Category { id, category } => {
            let mut note = storage
                .load_note(id)
                .map_err(|e| format!("加载笔记失败: {}", e))?;
            note.set_category(category);
            storage
                .save_note(&note)
                .map_err(|e| format!("保存笔记失败: {}", e))?;
            println!("📁 已设置分类: {}", category);
            Ok(())
        }

        Command::Export { format, output } => {
            let notes = storage
                .list_notes()
                .map_err(|e| format!("读取笔记列表失败: {}", e))?;

            let exporter = create_exporter(*format);

            if let Some(ref path_str) = output {
                let path = PathBuf::from(path_str);
                exporter
                    .export_to_file(&notes, &path)
                    .map_err(|e| format!("导出失败: {}", e))?;
                println!(
                    "📤 已导出 {} 条笔记到 {} (.{} 格式)",
                    notes.len(),
                    path.display(),
                    exporter.file_extension()
                );
            } else {
                let content = exporter
                    .export(&notes)
                    .map_err(|e| format!("导出失败: {}", e))?;
                println!(
                    "📤 导出 {} 条笔记 ({}):",
                    notes.len(),
                    exporter.format_name()
                );
                println!("{}", content);
            }
            Ok(())
        }

        Command::Stats => {
            let count = storage.count().map_err(|e| format!("统计失败: {}", e))?;
            let notes = storage
                .list_notes()
                .map_err(|e| format!("读取笔记列表失败: {}", e))?;
            let stats = compute_stats(&notes);
            println!("{}", stats);
            println!("  (存储目录: {})", storage.notes_dir().display());
            println!("  (通过 count() 确认: {} 条)", count);
            Ok(())
        }
    }
}

// 尝试打开编辑器
fn open_editor(path: &std::path::Path) -> Result<(), String> {
    let editors = ["code", "notepad", "vim", "nano"];

    for editor in &editors {
        let result = std::process::Command::new(editor).arg(path).spawn();

        match result {
            Ok(_) => {
                println!("📎 正在用 {} 打开...", editor);
                return Ok(());
            }
            Err(_) => continue,
        }
    }

    Err("找不到可用的编辑器，请手动编辑文件: ".to_string() + &path.to_string_lossy())
}
