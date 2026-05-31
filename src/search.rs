// 搜索和过滤 — 全文搜索、标签过滤、日期范围筛选
// 泛型过滤函数是这模块的重点

use crate::model::Note;
#[cfg(test)]
use chrono::Datelike;
use chrono::NaiveDate;
use regex::Regex;

// ---- 错误类型 ----

#[derive(Debug)]
pub enum SearchError {
    InvalidRegex(String),
    InvalidDate(String),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::InvalidRegex(msg) => write!(f, "正则表达式有误: {}", msg),
            SearchError::InvalidDate(msg) => write!(f, "日期格式不对: {}", msg),
        }
    }
}

// ---- 搜索条件 ----

// 用枚举组合不同的过滤方式，比一堆 bool 参数好维护
#[derive(Debug, Clone)]
pub enum Filter {
    ByTag(String),
    ByCategory(String),
    ByDateRange(NaiveDate, NaiveDate),
    #[allow(dead_code)]
    ByKeyword(String),
    #[allow(dead_code)]
    ByRegex(String),
    All(Vec<Filter>),  // 同时满足
    #[allow(dead_code)]
    Any(Vec<Filter>),  // 满足任一
}

impl Filter {
    pub fn matches(&self, note: &Note) -> bool {
        match self {
            Filter::ByTag(tag) => note.has_tag(tag),
            Filter::ByCategory(cat) => note
                .metadata
                .category
                .as_ref()
                .is_some_and(|c| c.to_lowercase() == cat.to_lowercase()),
            Filter::ByDateRange(start, end) => {
                note.metadata.modified >= *start && note.metadata.modified <= *end
            }
            Filter::ByKeyword(kw) => {
                let lower_kw = kw.to_lowercase();
                note.content.to_lowercase().contains(&lower_kw)
                    || note.metadata.title.to_lowercase().contains(&lower_kw)
            }
            Filter::ByRegex(pattern) => {
                // 每次都编译正则，效率不高但简单
                Regex::new(pattern)
                    .map(|re| re.is_match(&note.content) || re.is_match(&note.metadata.title))
                    .unwrap_or(false)
            }
            Filter::All(filters) => filters.iter().all(|f| f.matches(note)),
            Filter::Any(filters) => filters.iter().any(|f| f.matches(note)),
        }
    }
}

// ---- 泛型过滤 ----

// 传一个闭包进来做筛选，闭包/函数指针/啥都行
pub fn filter_notes<F>(notes: &[Note], predicate: F) -> Vec<&Note>
where
    F: Fn(&Note) -> bool,
{
    notes.iter().filter(|n| predicate(n)).collect()
}

pub fn filter_by_filter<'a>(notes: &'a [Note], filter: &Filter) -> Vec<&'a Note> {
    filter_notes(notes, |note| filter.matches(note))
}

// ---- 全文搜索 ----

pub struct SearchResult<'a> {
    pub note: &'a Note,
    pub matched_lines: Vec<(usize, String)>, // 行号 + 内容
}

// 关键词搜索，不区分大小写
pub fn search_keyword<'a>(notes: &'a [Note], keyword: &str) -> Vec<SearchResult<'a>> {
    let lower_kw = keyword.to_lowercase();
    let mut results = Vec::new();

    for note in notes {
        let matched_lines: Vec<(usize, String)> = note
            .content
            .lines()
            .enumerate()
            .filter(|(_, line)| line.to_lowercase().contains(&lower_kw))
            .map(|(i, line)| (i + 1, line.to_string()))
            .collect();

        let title_match = note.metadata.title.to_lowercase().contains(&lower_kw);

        if !matched_lines.is_empty() || title_match {
            results.push(SearchResult {
                note,
                matched_lines,
            });
        }
    }

    results
}

pub fn search_regex<'a>(
    notes: &'a [Note],
    pattern: &str,
) -> Result<Vec<SearchResult<'a>>, SearchError> {
    let re = Regex::new(pattern).map_err(|e| SearchError::InvalidRegex(e.to_string()))?;

    let mut results = Vec::new();

    for note in notes {
        let matched_lines: Vec<(usize, String)> = note
            .content
            .lines()
            .enumerate()
            .filter(|(_, line)| re.is_match(line))
            .map(|(i, line)| (i + 1, line.to_string()))
            .collect();

        let title_match = re.is_match(&note.metadata.title);

        if !matched_lines.is_empty() || title_match {
            results.push(SearchResult {
                note,
                matched_lines,
            });
        }
    }

    Ok(results)
}

// ---- 日期解析 ----

// 支持 "2026-05-31" 和 "20260531" 两种写法
pub fn parse_date_range(
    start_str: &str,
    end_str: &str,
) -> Result<(NaiveDate, NaiveDate), SearchError> {
    let start = NaiveDate::parse_from_str(start_str, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(start_str, "%Y%m%d"))
        .map_err(|_| SearchError::InvalidDate(format!("起始日期 '{}'", start_str)))?;

    let end = NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(end_str, "%Y%m%d"))
        .map_err(|_| SearchError::InvalidDate(format!("结束日期 '{}'", end_str)))?;

    Ok((start, end))
}

// ---- 测试 ----

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Note;

    fn make_test_notes() -> Vec<Note> {
        let mut n1 = Note::new("Rust 学习笔记");
        n1.add_tag("rust");
        n1.add_tag("programming");
        n1.set_content(
            "今天学习了 Rust 的所有权机制\nownership 是 Rust 的核心特性\n借用规则很重要",
        );

        let mut n2 = Note::new("Vue3 学习");
        n2.add_tag("vue");
        n2.add_tag("frontend");
        n2.set_content("Vue3 的组合式 API 很好用\nref 和 reactive 的区别\nRust 也可以写前端");

        let mut n3 = Note::new("每日总结");
        n3.add_tag("diary");
        n3.set_category("日常");
        n3.set_content("今天天气不错\n写了很多代码");

        vec![n1, n2, n3]
    }

    #[test]
    fn test_filter_by_tag() {
        let notes = make_test_notes();
        let filter = Filter::ByTag("rust".to_string());
        let result = filter_by_filter(&notes, &filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].metadata.title, "Rust 学习笔记");
    }

    #[test]
    fn test_filter_by_category() {
        let notes = make_test_notes();
        let filter = Filter::ByCategory("日常".to_string());
        let result = filter_by_filter(&notes, &filter);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_by_keyword() {
        let notes = make_test_notes();
        let filter = Filter::ByKeyword("Rust".to_string());
        let result = filter_by_filter(&notes, &filter);
        assert!(result.len() >= 2);
    }

    #[test]
    fn test_filter_all() {
        let notes = make_test_notes();
        let filter = Filter::All(vec![
            Filter::ByTag("rust".to_string()),
            Filter::ByKeyword("ownership".to_string()),
        ]);
        let result = filter_by_filter(&notes, &filter);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_any() {
        let notes = make_test_notes();
        let filter = Filter::Any(vec![
            Filter::ByTag("vue".to_string()),
            Filter::ByTag("diary".to_string()),
        ]);
        let result = filter_by_filter(&notes, &filter);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_generic_filter_function() {
        let notes = make_test_notes();
        // 传闭包做过滤
        let long_notes = filter_notes(&notes, |n| n.content.lines().count() > 2);
        assert!(long_notes.len() >= 1);
    }

    #[test]
    fn test_search_keyword() {
        let notes = make_test_notes();
        let results = search_keyword(&notes, "所有权");
        assert_eq!(results.len(), 1);
        assert!(!results[0].matched_lines.is_empty());
    }

    #[test]
    fn test_search_keyword_case_insensitive() {
        let notes = make_test_notes();
        let results = search_keyword(&notes, "rust");
        assert!(results.len() >= 2);
    }

    #[test]
    fn test_search_regex() {
        let notes = make_test_notes();
        let results = search_regex(&notes, r"ref|reactive").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_regex_invalid() {
        let notes = make_test_notes();
        let result = search_regex(&notes, r"[invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_date_range() {
        let (start, end) = parse_date_range("2026-01-01", "2026-12-31").unwrap();
        assert_eq!(start.year(), 2026);
        assert_eq!(end.month(), 12);
    }

    #[test]
    fn test_parse_date_range_compact() {
        let (start, end) = parse_date_range("20260101", "20261231").unwrap();
        assert_eq!(start.month(), 1);
        assert_eq!(end.month(), 12);
    }

    #[test]
    fn test_parse_date_range_invalid() {
        assert!(parse_date_range("bad", "2026-12-31").is_err());
    }
}
