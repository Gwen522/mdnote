// 文件存储 — 笔记的读写和 YAML front matter 解析
// 这块 ownership 和 Result 用得比较多

use crate::model::{Metadata, Note, Tag};
#[cfg(test)]
use chrono::Datelike;
use chrono::NaiveDate;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// ---- 错误类型 ----

#[derive(Debug)]
pub enum StorageError {
    NotFound(String),
    AlreadyExists(String),
    Io(io::Error),
    ParseError(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::NotFound(msg) => write!(f, "找不到: {}", msg),
            StorageError::AlreadyExists(msg) => write!(f, "已存在: {}", msg),
            StorageError::Io(err) => write!(f, "IO错误: {}", err),
            StorageError::ParseError(msg) => write!(f, "解析失败: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Io(err) => Some(err),
            _ => None,
        }
    }
}

// io::Error 自动转 StorageError，方便用 ?
impl From<io::Error> for StorageError {
    fn from(err: io::Error) -> Self {
        StorageError::Io(err)
    }
}

// ---- Storage 结构体 ----

pub struct Storage {
    notes_dir: PathBuf,
}

impl Storage {
    // 目录不存在就自动建
    pub fn new<P: AsRef<Path>>(notes_dir: P) -> Result<Self, StorageError> {
        let notes_dir = notes_dir.as_ref().to_path_buf();
        if !notes_dir.exists() {
            fs::create_dir_all(&notes_dir)?;
        }
        Ok(Self { notes_dir })
    }

    pub fn notes_dir(&self) -> &Path {
        &self.notes_dir
    }

    fn note_path(&self, id: &str) -> PathBuf {
        self.notes_dir.join(format!("{}.md", id))
    }

    // ---- 增删改查 ----

    pub fn create_note(&self, title: &str) -> Result<Note, StorageError> {
        let note = Note::new(title);
        let path = self.note_path(&note.id);

        if path.exists() {
            return Err(StorageError::AlreadyExists(format!(
                "笔记 '{}' 已经存在了",
                note.id
            )));
        }

        self.save_note(&note)?;
        Ok(note)
    }

    pub fn save_note(&self, note: &Note) -> Result<(), StorageError> {
        let path = self.note_path(&note.id);
        let content = format_note_file(&note.metadata, &note.content);
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn load_note(&self, id: &str) -> Result<Note, StorageError> {
        let path = self.note_path(id);
        if !path.exists() {
            return Err(StorageError::NotFound(format!("笔记 '{}'", id)));
        }

        let raw = fs::read_to_string(&path)?;
        parse_note_file(id, &raw, &path)
    }

    pub fn delete_note(&self, id: &str) -> Result<(), StorageError> {
        let path = self.note_path(id);
        if !path.exists() {
            return Err(StorageError::NotFound(format!("笔记 '{}'", id)));
        }
        fs::remove_file(&path)?;
        Ok(())
    }

    pub fn list_notes(&self) -> Result<Vec<Note>, StorageError> {
        if !self.notes_dir.exists() {
            return Ok(Vec::new());
        }

        let mut notes = Vec::new();
        let entries = fs::read_dir(&self.notes_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let raw = fs::read_to_string(&path)?;
                match parse_note_file(&id, &raw, &path) {
                    Ok(note) => notes.push(note),
                    Err(StorageError::ParseError(msg)) => {
                        eprintln!("警告: 跳过文件 {} - {}", id, msg);
                    }
                    Err(err) => return Err(err),
                }
            }
        }

        // 按修改时间倒序
        notes.sort_by(|a, b| b.metadata.modified.cmp(&a.metadata.modified));
        Ok(notes)
    }

    pub fn count(&self) -> Result<usize, StorageError> {
        let notes = self.list_notes()?;
        Ok(notes.len())
    }

    pub fn exists(&self, id: &str) -> bool {
        self.note_path(id).exists()
    }
}

// ---- YAML front matter 格式化与解析 ----

// 把笔记写成带 front matter 的 md 文件
fn format_note_file(metadata: &Metadata, content: &str) -> String {
    let tags_str = if metadata.tags.is_empty() {
        String::from("[]")
    } else {
        let items: Vec<String> = metadata
            .tags
            .iter()
            .map(|t| format!("\"{}\"", t.name))
            .collect();
        format!("[{}]", items.join(", "))
    };

    let category_str = metadata
        .category
        .as_deref()
        .map(|c| format!("\"{}\"", c))
        .unwrap_or_else(|| "null".to_string());

    format!(
        "---\ntitle: \"{}\"\ncreated: {}\nmodified: {}\ntags: {}\ncategory: {}\n---\n{}",
        metadata.title, metadata.created, metadata.modified, tags_str, category_str, content
    )
}

// 解析 md 文件，提取元信息和正文
fn parse_note_file(id: &str, raw: &str, filepath: &Path) -> Result<Note, StorageError> {
    let trimmed = raw.trim_start();
    if !trimmed.starts_with("---") {
        // 没有 front matter，整篇当正文
        return Ok(Note {
            id: id.to_string(),
            metadata: Metadata::new(id),
            content: raw.to_string(),
            filepath: filepath.to_path_buf(),
        });
    }

    // 找第二个 ---
    let rest = &trimmed[3..];
    let end_pos = rest
        .find("\n---")
        .ok_or_else(|| StorageError::ParseError("front matter 没有结束标记".to_string()))?;

    let front_matter = &rest[..end_pos];
    let content_start = end_pos + 4;
    let content = if content_start < rest.len() {
        rest[content_start..].trim_start().to_string()
    } else {
        String::new()
    };

    let metadata = parse_front_matter(front_matter)?;

    Ok(Note {
        id: id.to_string(),
        metadata,
        content,
        filepath: filepath.to_path_buf(),
    })
}

// 解析 front matter 文本
fn parse_front_matter(text: &str) -> Result<Metadata, StorageError> {
    let mut title = String::from("无标题");
    let mut created = chrono::Local::now().date_naive();
    let mut modified = created;
    let mut tags: Vec<Tag> = Vec::new();
    let mut category: Option<String> = None;

    for line in text.lines() {
        let line = line.trim();

        if let Some(val) = line.strip_prefix("title:") {
            title = parse_quoted_value(val.trim());
        } else if let Some(val) = line.strip_prefix("created:") {
            created = parse_date(val.trim())?;
        } else if let Some(val) = line.strip_prefix("modified:") {
            modified = parse_date(val.trim())?;
        } else if let Some(val) = line.strip_prefix("category:") {
            let v = val.trim();
            if v != "null" && !v.is_empty() {
                category = Some(parse_quoted_value(v));
            }
        }
        // tags 只处理单行 [] 格式，多行的懒得写了
        else if let Some(val) = line.strip_prefix("tags:") {
            let v = val.trim();
            if v.starts_with('[') {
                tags = parse_inline_tags(v);
            }
        } else if line.starts_with('"') && line.ends_with('"') && !tags.is_empty() {
            // 可能是 tags 数组里的一项
            let tag_name = line.trim_matches('"').to_string();
            tags.push(Tag::new(&tag_name));
        }
    }

    Ok(Metadata {
        title,
        created,
        modified,
        tags,
        category,
    })
}

// "hello" -> hello
fn parse_quoted_value(s: &str) -> String {
    s.trim().trim_matches('"').to_string()
}

fn parse_date(s: &str) -> Result<NaiveDate, StorageError> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| StorageError::ParseError(format!("日期格式不对 '{}': {}", s, e)))
}

// 解析 ["rust", "cli"] 这种格式
fn parse_inline_tags(s: &str) -> Vec<Tag> {
    let inner = s.trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .map(|item| item.trim().trim_matches('"'))
        .filter(|item| !item.is_empty())
        .map(Tag::new)
        .collect()
}

// ---- 测试 ----

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_note_file() {
        let mut meta = Metadata::new("测试笔记");
        meta.add_tag(Tag::new("rust"));
        meta.category = Some("学习".to_string());
        let content = "这是正文内容";
        let result = format_note_file(&meta, content);

        assert!(result.starts_with("---"));
        assert!(result.contains("title: \"测试笔记\""));
        assert!(result.contains("rust"));
        assert!(result.contains("category: \"学习\""));
        assert!(result.contains("这是正文内容"));
    }

    #[test]
    fn test_parse_quoted_value() {
        assert_eq!(parse_quoted_value("\"hello\""), "hello");
        assert_eq!(parse_quoted_value("  \"world\"  "), "world");
        assert_eq!(parse_quoted_value("plain"), "plain");
    }

    #[test]
    fn test_parse_date() {
        let d = parse_date("2026-05-31").unwrap();
        assert_eq!(d.year(), 2026);
        assert_eq!(d.month(), 5);
        assert_eq!(d.day(), 31);
    }

    #[test]
    fn test_parse_date_invalid() {
        assert!(parse_date("not-a-date").is_err());
    }

    #[test]
    fn test_parse_inline_tags() {
        let tags = parse_inline_tags("[\"rust\", \"cli\"]");
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "rust");
        assert_eq!(tags[1].name, "cli");
    }

    #[test]
    fn test_parse_inline_tags_empty() {
        let tags = parse_inline_tags("[]");
        assert!(tags.is_empty());
    }

    #[test]
    fn test_parse_note_file_with_front_matter() {
        let raw = "---\ntitle: \"我的笔记\"\ncreated: 2026-05-31\nmodified: 2026-05-31\ntags: [\"rust\"]\ncategory: null\n---\n笔记正文";
        let result = parse_note_file("my-note", raw, PathBuf::new().as_path());
        assert!(result.is_ok());
        let note = result.unwrap();
        assert_eq!(note.metadata.title, "我的笔记");
        assert_eq!(note.metadata.tags.len(), 1);
        assert_eq!(note.metadata.tags[0].name, "rust");
        assert_eq!(note.content, "笔记正文");
    }

    #[test]
    fn test_parse_note_file_without_front_matter() {
        let raw = "直接就是正文内容";
        let result = parse_note_file("test", raw, PathBuf::new().as_path());
        assert!(result.is_ok());
        let note = result.unwrap();
        assert_eq!(note.content, "直接就是正文内容");
    }

    #[test]
    fn test_storage_create_and_load() {
        let tmp = std::env::temp_dir().join("mdnote_test_create");
        let _ = fs::remove_dir_all(&tmp);
        let storage = Storage::new(&tmp).unwrap();

        let note = storage.create_note("Test Create Note").unwrap();
        assert_eq!(note.id, "test-create-note");

        let loaded = storage.load_note("test-create-note").unwrap();
        assert_eq!(loaded.metadata.title, "Test Create Note");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_storage_duplicate_create() {
        let tmp = std::env::temp_dir().join("mdnote_test_dup");
        let _ = fs::remove_dir_all(&tmp);
        let storage = Storage::new(&tmp).unwrap();

        storage.create_note("Duplicate Test").unwrap();
        let result = storage.create_note("Duplicate Test");
        assert!(matches!(result, Err(StorageError::AlreadyExists(_))));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_storage_delete_note() {
        let tmp = std::env::temp_dir().join("mdnote_test_delete");
        let _ = fs::remove_dir_all(&tmp);
        let storage = Storage::new(&tmp).unwrap();

        storage.create_note("Pending Delete Note").unwrap();
        assert!(storage.exists("pending-delete-note"));

        storage.delete_note("pending-delete-note").unwrap();
        assert!(!storage.exists("pending-delete-note"));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_storage_list_notes() {
        let tmp = std::env::temp_dir().join("mdnote_test_list");
        let _ = fs::remove_dir_all(&tmp);
        let storage = Storage::new(&tmp).unwrap();

        storage.create_note("笔记A").unwrap();
        storage.create_note("笔记B").unwrap();

        let notes = storage.list_notes().unwrap();
        assert_eq!(notes.len(), 2);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_storage_load_nonexistent() {
        let tmp = std::env::temp_dir().join("mdnote_test_404");
        let _ = fs::remove_dir_all(&tmp);
        let storage = Storage::new(&tmp).unwrap();

        let result = storage.load_note("no-such-note");
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }
}
