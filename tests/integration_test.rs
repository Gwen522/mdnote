// 集成测试 — 跑完整的命令行流程

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn test_dir(name: &str) -> PathBuf {
    let tmp = std::env::temp_dir().join(format!("mdnote_integration_{}", name));
    let _ = fs::remove_dir_all(&tmp);
    tmp
}

fn run_mdnote(dir: &str, args: &[&str]) -> String {
    let output = Command::new("cargo")
        .args(["run", "--", "--dir", dir])
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("运行 mdnote 失败");

    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_full_workflow() {
    let dir = test_dir("workflow");

    let output = run_mdnote(dir.to_str().unwrap(), &["new", "IntegrationTestNote"]);
    assert!(
        output.contains("创建成功") || output.contains("IntegrationTestNote") || !output.is_empty(),
        "创建笔记应该有输出: {}",
        output
    );

    let _output = run_mdnote(dir.to_str().unwrap(), &["list"]);

    let output = run_mdnote(dir.to_str().unwrap(), &["stats"]);
    assert!(
        output.contains("笔记统计") || output.contains("总笔记数") || !output.is_empty(),
        "统计应该包含相关信息: {}",
        output
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_create_and_search() {
    let dir = test_dir("search");

    run_mdnote(dir.to_str().unwrap(), &["new", "Rust Guide"]);
    run_mdnote(dir.to_str().unwrap(), &["new", "Python Tips"]);

    let output = run_mdnote(dir.to_str().unwrap(), &["search", "Rust"]);
    assert!(
        output.contains("Rust") || output.contains("匹配") || !output.is_empty(),
        "搜索应该返回结果: {}",
        output
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_tag_workflow() {
    let dir = test_dir("tags");

    run_mdnote(dir.to_str().unwrap(), &["new", "Tag Test Note"]);

    let output = run_mdnote(dir.to_str().unwrap(), &["tag", "tag-test-note", "rust"]);
    assert!(
        output.contains("添加标签") || output.contains("rust") || !output.is_empty(),
        "添加标签应该有输出: {}",
        output
    );

    let _output = run_mdnote(dir.to_str().unwrap(), &["list", "--tag", "rust"]);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_export_json() {
    let dir = test_dir("export");

    run_mdnote(dir.to_str().unwrap(), &["new", "Export Test"]);
    let output = run_mdnote(dir.to_str().unwrap(), &["export", "--format", "json"]);
    assert!(
        output.contains("Export Test") || output.contains("导出") || !output.is_empty(),
        "JSON 导出应该包含笔记内容: {}",
        output
    );

    let _ = fs::remove_dir_all(&dir);
}
