// 数据模型 — 笔记、标签、命令这些核心结构

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

// ---- 核心结构体 ----

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    pub title: String,
    pub created: NaiveDate,
    pub modified: NaiveDate,
    pub tags: Vec<Tag>,
    pub category: Option<String>,
}

impl Metadata {
    pub fn new(title: &str) -> Self {
        let today = chrono::Local::now().date_naive();
        Self {
            title: title.to_string(),
            created: today,
            modified: today,
            tags: Vec::new(),
            category: None,
        }
    }

    // 加标签，重复的不会加
    pub fn add_tag(&mut self, tag: Tag) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn remove_tag(&mut self, tag_name: &str) {
        self.tags.retain(|t| t.name != tag_name);
    }

    // 更新修改日期
    pub fn touch(&mut self) {
        self.modified = chrono::Local::now().date_naive();
    }
}

// 标签 — 根据 name 判等
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Tag {
    pub name: String,
}

impl Tag {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_lowercase().trim().to_string(),
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: String, // 就是文件名去掉 .md
    pub metadata: Metadata,
    pub content: String,
    #[serde(skip)]
    pub filepath: std::path::PathBuf,
}

impl Note {
    pub fn new(title: &str) -> Self {
        let id = title_to_id(title);
        Self {
            id,
            metadata: Metadata::new(title),
            content: String::new(),
            filepath: PathBuf::new(),
        }
    }

    pub fn set_category(&mut self, category: &str) {
        self.metadata.category = Some(category.to_string());
        self.metadata.touch();
    }

    pub fn add_tag(&mut self, tag_name: &str) {
        self.metadata.add_tag(Tag::new(tag_name));
        self.metadata.touch();
    }

    pub fn remove_tag(&mut self, tag_name: &str) {
        self.metadata.remove_tag(tag_name);
        self.metadata.touch();
    }

    pub fn tag_names(&self) -> Vec<&str> {
        self.metadata.tags.iter().map(|t| t.name.as_str()).collect()
    }

    pub fn has_tag(&self, tag_name: &str) -> bool {
        let lower = tag_name.to_lowercase();
        self.metadata.tags.iter().any(|t| t.name == lower)
    }

    #[allow(dead_code)]
    pub fn set_content(&mut self, content: &str) {
        self.content = content.to_string();
        self.metadata.touch();
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tags_str = if self.metadata.tags.is_empty() {
            String::from("(无标签)")
        } else {
            self.metadata
                .tags
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        };

        let category = self.metadata.category.as_deref().unwrap_or("(未分类)");

        writeln!(f, "────────────────────────────────")?;
        writeln!(f, "  标题: {}", self.metadata.title)?;
        writeln!(f, "  ID:   {}", self.id)?;
        writeln!(f, "  分类: {}", category)?;
        writeln!(f, "  标签: {}", tags_str)?;
        writeln!(
            f,
            "  创建: {}  |  修改: {}",
            self.metadata.created, self.metadata.modified
        )?;
        writeln!(f, "────────────────────────────────")?;
        if !self.content.is_empty() {
            // 只显示前5行
            let preview: Vec<&str> = self.content.lines().take(5).collect();
            for line in preview {
                writeln!(f, "  {}", line)?;
            }
            let total_lines = self.content.lines().count();
            if total_lines > 5 {
                writeln!(f, "  ... (共 {} 行)", total_lines)?;
            }
        }
        Ok(())
    }
}

// ---- 命令枚举 ----

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    New { title: String },
    List {
        tag: Option<String>,
        category: Option<String>,
        date: Option<String>,
        date_start: Option<String>,
        date_end: Option<String>,
    },
    Search { keyword: String, regex: bool },
    Show { id: String },
    Edit { id: String },
    Delete { id: String },
    Tag { id: String, tag: String },
    Untag { id: String, tag: String },
    Category { id: String, category: String },
    Export {
        format: ExportFormat,
        output: Option<String>,
    },
    Stats,
}

// ---- 导出格式 ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Html,
    Text,
}

impl ExportFormat {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(ExportFormat::Json),
            "html" => Some(ExportFormat::Html),
            "text" | "txt" => Some(ExportFormat::Text),
            _ => None,
        }
    }
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportFormat::Json => write!(f, "json"),
            ExportFormat::Html => write!(f, "html"),
            ExportFormat::Text => write!(f, "text"),
        }
    }
}

// ---- 工具函数 ----

// 把标题转成文件名用的 id：小写、空格变横线、去特殊字符
fn title_to_id(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else if c.is_whitespace() {
                '-'
            } else {
                '\0' // 特殊字符标记为删除
            }
        })
        .filter(|c| *c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

// ---- 测试 ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("Rust");
        assert_eq!(tag.name, "rust");
        assert_eq!(format!("{}", tag), "#rust");
    }

    #[test]
    fn test_tag_equality() {
        let t1 = Tag::new("hello");
        let t2 = Tag::new("Hello");
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_metadata_add_tag_no_duplicate() {
        let mut meta = Metadata::new("测试笔记");
        meta.add_tag(Tag::new("rust"));
        meta.add_tag(Tag::new("Rust")); // 重复的
        assert_eq!(meta.tags.len(), 1);
    }

    #[test]
    fn test_metadata_remove_tag() {
        let mut meta = Metadata::new("测试笔记");
        meta.add_tag(Tag::new("rust"));
        meta.add_tag(Tag::new("vue"));
        meta.remove_tag("rust");
        assert_eq!(meta.tags.len(), 1);
        assert_eq!(meta.tags[0].name, "vue");
    }

    #[test]
    fn test_title_to_id() {
        assert_eq!(title_to_id("Hello World"), "hello-world");
        // 中文在 is_alphanumeric 里算 true，会保留
        assert_eq!(title_to_id("Rust 学习笔记"), "rust-学习笔记");
        assert_eq!(title_to_id("test---multiple"), "test-multiple");
        assert_eq!(title_to_id("  spaces  "), "spaces");
    }

    #[test]
    fn test_note_new() {
        let note = Note::new("My First Note");
        assert_eq!(note.id, "my-first-note");
        assert_eq!(note.metadata.title, "My First Note");
        assert!(note.content.is_empty());
    }

    #[test]
    fn test_note_add_remove_tag() {
        let mut note = Note::new("Test Note");
        note.add_tag("rust");
        note.add_tag("cli");
        assert!(note.has_tag("rust"));
        assert!(note.has_tag("RUST"));
        note.remove_tag("rust");
        assert!(!note.has_tag("rust"));
        assert_eq!(note.tag_names(), vec!["cli"]);
    }

    #[test]
    fn test_note_set_category() {
        let mut note = Note::new("Test");
        assert!(note.metadata.category.is_none());
        note.set_category("学习");
        assert_eq!(note.metadata.category.as_deref(), Some("学习"));
    }

    #[test]
    fn test_export_format_parse() {
        assert_eq!(ExportFormat::from_str_opt("json"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::from_str_opt("HTML"), Some(ExportFormat::Html));
        assert_eq!(ExportFormat::from_str_opt("txt"), Some(ExportFormat::Text));
        assert_eq!(ExportFormat::from_str_opt("pdf"), None);
    }

    #[test]
    fn test_note_display() {
        let mut note = Note::new("展示测试");
        note.add_tag("test");
        note.set_content("第一行\n第二行\n第三行\n第四行\n第五行\n第六行");
        let output = format!("{}", note);
        assert!(output.contains("展示测试"));
        assert!(output.contains("#test"));
        assert!(output.contains("第一行"));
    }
}
