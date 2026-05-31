// 导出模块 — Exportable trait 和 JSON/HTML/纯文本三种导出

use crate::model::Note;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

// ---- 错误类型 ----

#[derive(Debug)]
pub enum ExportError {
    Io(io::Error),
    NoNotes,
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::Io(err) => write!(f, "写入失败: {}", err),
            ExportError::NoNotes => write!(f, "没有笔记可以导出"),
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExportError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ExportError {
    fn from(err: io::Error) -> Self {
        ExportError::Io(err)
    }
}

// ---- Exportable trait ----

// 实现了这个 trait 就能导出笔记，具体导出成啥格式由实现方决定
pub trait Exportable {
    fn export(&self, notes: &[Note]) -> Result<String, ExportError>;
    fn format_name(&self) -> &str;
    fn file_extension(&self) -> &str;

    // 默认实现：导出到文件
    fn export_to_file(&self, notes: &[Note], path: &Path) -> Result<(), ExportError> {
        if notes.is_empty() {
            return Err(ExportError::NoNotes);
        }
        let content = self.export(notes)?;
        fs::write(path, content)?;
        Ok(())
    }
}

// ---- JSON 导出 ----

pub struct JsonExporter;

impl Exportable for JsonExporter {
    fn export(&self, notes: &[Note]) -> Result<String, ExportError> {
        if notes.is_empty() {
            return Err(ExportError::NoNotes);
        }
        serde_json::to_string_pretty(notes)
            .map_err(|e| ExportError::Io(io::Error::other(e.to_string())))
    }

    fn format_name(&self) -> &str {
        "JSON"
    }

    fn file_extension(&self) -> &str {
        "json"
    }
}

// ---- HTML 导出 ----

pub struct HtmlExporter;

impl Exportable for HtmlExporter {
    fn export(&self, notes: &[Note]) -> Result<String, ExportError> {
        if notes.is_empty() {
            return Err(ExportError::NoNotes);
        }

        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n");
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str("  <title>mdnote 笔记导出</title>\n");
        html.push_str("  <style>\n");
        html.push_str("    body { font-family: sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }\n");
        html.push_str("    .note { border: 1px solid #ddd; border-radius: 8px; padding: 16px; margin-bottom: 16px; }\n");
        html.push_str("    .note h2 { margin-top: 0; color: #333; }\n");
        html.push_str("    .meta { color: #666; font-size: 0.9em; }\n");
        html.push_str("    .tags span { background: #e0e0e0; padding: 2px 8px; border-radius: 4px; margin-right: 4px; }\n");
        html.push_str("    pre { background: #f5f5f5; padding: 12px; border-radius: 4px; overflow-x: auto; }\n");
        html.push_str("  </style>\n");
        html.push_str("</head>\n<body>\n");
        html.push_str("  <h1>mdnote 笔记导出</h1>\n");

        for note in notes {
            html.push_str("  <div class=\"note\">\n");
            html.push_str(&format!(
                "    <h2>{}</h2>\n",
                escape_html(&note.metadata.title)
            ));

            html.push_str("    <div class=\"meta\">\n");
            html.push_str(&format!(
                "      创建: {} | 修改: {}",
                note.metadata.created, note.metadata.modified
            ));

            if let Some(cat) = &note.metadata.category {
                html.push_str(&format!(" | 分类: {}", escape_html(cat)));
            }
            html.push_str("\n    </div>\n");

            if !note.metadata.tags.is_empty() {
                html.push_str("    <div class=\"tags\">\n");
                for tag in &note.metadata.tags {
                    html.push_str(&format!("      <span>{}</span>\n", escape_html(&tag.name)));
                }
                html.push_str("    </div>\n");
            }

            if !note.content.is_empty() {
                html.push_str("    <pre>");
                html.push_str(&escape_html(&note.content));
                html.push_str("</pre>\n");
            }

            html.push_str("  </div>\n");
        }

        html.push_str("</body>\n</html>\n");
        Ok(html)
    }

    fn format_name(&self) -> &str {
        "HTML"
    }

    fn file_extension(&self) -> &str {
        "html"
    }
}

// ---- 纯文本导出 ----

pub struct TextExporter;

impl Exportable for TextExporter {
    fn export(&self, notes: &[Note]) -> Result<String, ExportError> {
        if notes.is_empty() {
            return Err(ExportError::NoNotes);
        }

        let mut text = String::new();
        for (i, note) in notes.iter().enumerate() {
            if i > 0 {
                text.push_str("\n================================\n\n");
            }
            text.push_str(&format!("{}", note));
        }
        Ok(text)
    }

    fn format_name(&self) -> &str {
        "Text"
    }

    fn file_extension(&self) -> &str {
        "txt"
    }
}

// ---- 辅助函数 ----

// 防 XSS
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// 用 trait object 做动态分发的导出函数
#[allow(dead_code)]
pub fn export_notes(
    exporter: &dyn Exportable,
    notes: &[Note],
    output_path: Option<&Path>,
) -> Result<String, ExportError> {
    let content = exporter.export(notes)?;

    if let Some(path) = output_path {
        fs::write(path, &content)?;
    }

    Ok(content)
}

// 根据格式创建导出器，返回 Box<dyn Exportable>
pub fn create_exporter(format: crate::model::ExportFormat) -> Box<dyn Exportable> {
    match format {
        crate::model::ExportFormat::Json => Box::new(JsonExporter),
        crate::model::ExportFormat::Html => Box::new(HtmlExporter),
        crate::model::ExportFormat::Text => Box::new(TextExporter),
    }
}

// ---- 统计信息 ----

#[derive(Debug)]
pub struct Stats {
    pub total_notes: usize,
    pub total_tags: usize,
    pub categories: Vec<String>,
    pub tag_counts: Vec<(String, usize)>,
    pub newest_date: Option<String>,
    pub oldest_date: Option<String>,
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "📊 笔记统计")?;
        writeln!(f, "───────────────────")?;
        writeln!(f, "  总笔记数: {}", self.total_notes)?;
        writeln!(f, "  标签总数: {}", self.total_tags)?;
        writeln!(f, "  分类数量: {}", self.categories.len())?;

        if !self.tag_counts.is_empty() {
            writeln!(f, "\n  标签使用排行:")?;
            for (tag, count) in &self.tag_counts {
                writeln!(f, "    #{} × {}", tag, count)?;
            }
        }

        if let (Some(oldest), Some(newest)) = (&self.oldest_date, &self.newest_date) {
            writeln!(f, "\n  最早笔记: {}", oldest)?;
            writeln!(f, "  最新笔记: {}", newest)?;
        }

        Ok(())
    }
}

pub fn compute_stats(notes: &[Note]) -> Stats {
    use std::collections::HashMap;

    let mut tag_map: HashMap<String, usize> = HashMap::new();
    let mut categories: Vec<String> = Vec::new();

    for note in notes {
        for tag in &note.metadata.tags {
            *tag_map.entry(tag.name.clone()).or_insert(0) += 1;
        }
        if let Some(cat) = &note.metadata.category {
            if !categories.contains(cat) {
                categories.push(cat.clone());
            }
        }
    }

    let mut tag_counts: Vec<(String, usize)> = tag_map.into_iter().collect();
    tag_counts.sort_by(|a, b| b.1.cmp(&a.1));

    let newest_date = notes.first().map(|n| n.metadata.modified.to_string());
    let oldest_date = notes.last().map(|n| n.metadata.modified.to_string());

    Stats {
        total_notes: notes.len(),
        total_tags: tag_counts.len(),
        categories,
        tag_counts,
        newest_date,
        oldest_date,
    }
}

// ---- 测试 ----

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Note;

    fn make_test_notes() -> Vec<Note> {
        let mut n1 = Note::new("笔记一");
        n1.add_tag("rust");
        n1.add_tag("cli");
        n1.set_content("这是第一条笔记");

        let mut n2 = Note::new("笔记二");
        n2.add_tag("rust");
        n2.set_category("学习");
        n2.set_content("这是第二条笔记\n多一行");

        vec![n1, n2]
    }

    #[test]
    fn test_json_export() {
        let notes = make_test_notes();
        let exporter = JsonExporter;
        let result = exporter.export(&notes);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("笔记一"));
        assert!(json.contains("rust"));
    }

    #[test]
    fn test_html_export() {
        let mut notes = make_test_notes();
        notes[0].set_content("一些内容 <b>加粗</b>");
        let exporter = HtmlExporter;
        let result = exporter.export(&notes);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("&lt;b&gt;"));
    }

    #[test]
    fn test_text_export() {
        let notes = make_test_notes();
        let exporter = TextExporter;
        let result = exporter.export(&notes);
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("笔记一"));
        assert!(text.contains("笔记二"));
    }

    #[test]
    fn test_export_empty() {
        let notes: Vec<Note> = Vec::new();
        let exporter = JsonExporter;
        let result = exporter.export(&notes);
        assert!(matches!(result, Err(ExportError::NoNotes)));
    }

    #[test]
    fn test_export_to_file() {
        let notes = make_test_notes();
        let exporter = TextExporter;
        let tmp = std::env::temp_dir().join("mdnote_export_test.txt");
        let result = exporter.export_to_file(&notes, &tmp);
        assert!(result.is_ok());
        assert!(tmp.exists());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_export_via_trait_object() {
        let notes = make_test_notes();
        let exporter: Box<dyn Exportable> = create_exporter(crate::model::ExportFormat::Json);
        let result = exporter.export(&notes);
        assert!(result.is_ok());
        assert_eq!(exporter.format_name(), "JSON");
        assert_eq!(exporter.file_extension(), "json");
    }

    #[test]
    fn test_export_notes_function() {
        let notes = make_test_notes();
        let result = export_notes(&JsonExporter, &notes, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(
            escape_html("<script>test</script>"),
            "&lt;script&gt;test&lt;/script&gt;"
        );
        assert_eq!(escape_html("a & b < c > d"), "a &amp; b &lt; c &gt; d");
        assert_eq!(escape_html("say \"hello\""), "say &quot;hello&quot;");
    }

    #[test]
    fn test_compute_stats() {
        let notes = make_test_notes();
        let stats = compute_stats(&notes);
        assert_eq!(stats.total_notes, 2);
        assert!(stats.total_tags >= 2);
        assert_eq!(stats.tag_counts[0].0, "rust");
    }

    #[test]
    fn test_create_exporter_all_formats() {
        let json = create_exporter(crate::model::ExportFormat::Json);
        assert_eq!(json.format_name(), "JSON");

        let html = create_exporter(crate::model::ExportFormat::Html);
        assert_eq!(html.format_name(), "HTML");

        let text = create_exporter(crate::model::ExportFormat::Text);
        assert_eq!(text.format_name(), "Text");
    }
}
