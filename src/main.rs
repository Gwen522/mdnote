// mdnote — Markdown 笔记管理工具
// 新建/列出/搜索/编辑/删除笔记，标签管理，多格式导出，统计信息

mod cli;
mod export;
mod model;
mod search;
mod storage;

use clap::Parser;

fn main() {
    let args = cli::Cli::parse();

    let storage = match storage::Storage::new(&args.dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ 初始化存储目录失败: {}", e);
            std::process::exit(1);
        }
    };

    let command = cli::parse_command(&args);

    match command {
        Some(cmd) => {
            if let Err(e) = cli::execute(&cmd, &storage) {
                eprintln!("❌ {}", e);
                std::process::exit(1);
            }
        }
        None => {
            println!("mdnote — Markdown 笔记管理工具");
            println!("用 --help 查看可用命令");
        }
    }
}
