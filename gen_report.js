const fs = require("fs");
const {
  Document, Packer, Paragraph, TextRun, ImageRun, Table, TableRow, TableCell,
  AlignmentType, HeadingLevel, BorderStyle, WidthType, ShadingType,
  PageBreak, Header, Footer, PageNumber, LevelFormat
} = require("docx");

// 读取校徽图片
const logoPath = "E:/2026DaSanXiaHomework/2026RustFinalWork/template_unpacked/word/media/image1.png";
const logoData = fs.readFileSync(logoPath);

// 公用的格式常量
const FONT_SONG = { ascii: "宋体", hAnsi: "宋体", eastAsia: "宋体", cs: "宋体" };
const SZ_TITLE = 44;   // 封面标题
const SZ_H1 = 36;      // 一级标题
const SZ_BODY = 24;     // 正文
const SZ_SMALL = 21;    // 小字

// 构造正文段落（首行缩进2字符，两端对齐）
function bodyPara(text, opts = {}) {
  return new Paragraph({
    indent: { firstLine: 480 },
    alignment: AlignmentType.JUSTIFIED,
    spacing: { line: 360 },
    ...opts,
    children: [
      new TextRun({
        text,
        font: FONT_SONG,
        size: SZ_BODY,
        ...(opts.runOpts || {})
      })
    ]
  });
}

// 构造一级标题
function h1(text) {
  return new Paragraph({
    spacing: { before: 200, after: 100 },
    outlineLevel: 1,
    children: [
      new TextRun({
        text,
        font: FONT_SONG,
        size: SZ_H1,
        bold: true,
      })
    ]
  });
}

// 构造二级标题
function h2(text) {
  return new Paragraph({
    spacing: { before: 160, after: 80 },
    indent: { firstLine: 240 },
    children: [
      new TextRun({
        text,
        font: FONT_SONG,
        size: 28,
        bold: true,
      })
    ]
  });
}

// 空行
function emptyLine() {
  return new Paragraph({ children: [] });
}

// 代码块段落（左对齐，等宽字体）
function codeLine(text) {
  return new Paragraph({
    spacing: { line: 276 },
    indent: { left: 480 },
    children: [
      new TextRun({
        text,
        font: { ascii: "Consolas", hAnsi: "Consolas", eastAsia: "宋体" },
        size: 20,
      })
    ]
  });
}

// 构造简单表格
function makeTable(headers, rows) {
  const border = { style: BorderStyle.SINGLE, size: 1, color: "999999" };
  const borders = { top: border, bottom: border, left: border, right: border };
  const colCount = headers.length;
  const colWidth = Math.floor(8306 / colCount);  // A4 内容区宽度大约8306 DXA
  const colWidths = headers.map(() => colWidth);
  const tableWidth = colWidth * colCount;

  const headerRow = new TableRow({
    children: headers.map(h => new TableCell({
      borders,
      width: { size: colWidth, type: WidthType.DXA },
      shading: { fill: "E8E8E8", type: ShadingType.CLEAR },
      margins: { top: 60, bottom: 60, left: 100, right: 100 },
      children: [new Paragraph({
        alignment: AlignmentType.CENTER,
        children: [new TextRun({ text: h, font: FONT_SONG, size: SZ_BODY, bold: true })]
      })]
    }))
  });

  const dataRows = rows.map(row => new TableRow({
    children: row.map(cell => new TableCell({
      borders,
      width: { size: colWidth, type: WidthType.DXA },
      margins: { top: 60, bottom: 60, left: 100, right: 100 },
      children: [new Paragraph({
        children: [new TextRun({ text: cell, font: FONT_SONG, size: SZ_BODY })]
      })]
    }))
  }));

  return new Table({
    width: { size: tableWidth, type: WidthType.DXA },
    columnWidths: colWidths,
    rows: [headerRow, ...dataRows]
  });
}

async function main() {
  const doc = new Document({
    styles: {
      default: {
        document: { run: { font: "宋体", size: SZ_BODY } }
      }
    },
    sections: [
      // ====== 封面 ======
      {
        properties: {
          page: {
            size: { width: 11906, height: 16838 },
            margin: { top: 1440, right: 1800, bottom: 1440, left: 1800 }
          }
        },
        children: [
          // 校徽
          new Paragraph({
            alignment: AlignmentType.CENTER,
            children: [
              new ImageRun({
                type: "png",
                data: logoData,
                transformation: { width: 245, height: 50 },
                altText: { title: "校徽", description: "校徽", name: "logo" }
              })
            ]
          }),
          emptyLine(),
          // 标题
          new Paragraph({
            alignment: AlignmentType.CENTER,
            spacing: { before: 200 },
            children: [
              new TextRun({ text: "Rust", font: FONT_SONG, size: SZ_TITLE }),
              new TextRun({ text: "课程作业报告", font: FONT_SONG, size: SZ_TITLE }),
            ]
          }),
          emptyLine(),
          emptyLine(),
          emptyLine(),
          emptyLine(),
          emptyLine(),
          // 项目名称
          new Paragraph({
            alignment: AlignmentType.CENTER,
            spacing: { before: 200 },
            children: [
              new TextRun({ text: "mdnote — Markdown笔记管理工具", font: FONT_SONG, size: SZ_H1 }),
            ]
          }),
          emptyLine(),
          emptyLine(),
          emptyLine(),
          emptyLine(),
          emptyLine(),
          emptyLine(),
          // 姓名
          new Paragraph({
            alignment: AlignmentType.CENTER,
            spacing: { line: 480 },
            children: [
              new TextRun({ text: "姓名：", font: FONT_SONG, size: SZ_H1 }),
            ]
          }),
          // 学号
          new Paragraph({
            alignment: AlignmentType.CENTER,
            spacing: { line: 480 },
            children: [
              new TextRun({ text: "学号：", font: FONT_SONG, size: SZ_H1 }),
            ]
          }),
          emptyLine(),
          emptyLine(),
          // 日期
          new Paragraph({
            alignment: AlignmentType.CENTER,
            children: [
              new TextRun({ text: "2026 年 5 月 31 日", font: FONT_SONG, size: 28 }),
            ]
          }),
        ]
      },

      // ====== 正文 ======
      {
        properties: {
          page: {
            size: { width: 11906, height: 16838 },
            margin: { top: 1440, right: 1800, bottom: 1440, left: 1800 }
          }
        },
        children: [
          // 一、项目简介
          h1("一、项目简介"),
          bodyPara("mdnote 是一个用 Rust 写的命令行 Markdown 笔记管理工具。主要功能就是在命令行里管理你的 .md 笔记文件——新建、编辑、删除、搜索、按标签分类筛选、导出成不同格式这些。笔记以 .md 文件存放在本地 notes/ 目录下，每个文件自带 YAML front matter 来记录元信息（标题、创建时间、标签、分类等）。"),
          bodyPara("GitHub 仓库地址：https://github.com/Gwen522/mdnote"),

          // 二、小组成员分工 — 删掉（单人开发）
          // 直接跳到三

          // 三、项目结构
          h1("二、项目结构"),   // 因为删了第二章所以重新编号
          bodyPara("项目总共 5 个模块，加上 main.rs 和测试文件，结构如下："),
          emptyLine(),
          codeLine("mdnote/"),
          codeLine("├── Cargo.toml          # 依赖配置"),
          codeLine("├── README.md           # 使用说明"),
          codeLine("├── src/"),
          codeLine("│   ├── main.rs         # 程序入口"),
          codeLine("│   ├── cli.rs          # 命令行参数解析、命令路由"),
          codeLine("│   ├── model.rs        # 数据模型（Note, Tag, Metadata, Command枚举等）"),
          codeLine("│   ├── storage.rs      # 文件读写、YAML front matter 解析"),
          codeLine("│   ├── search.rs       # 搜索、过滤、日期范围筛选"),
          codeLine("│   └── export.rs       # Exportable trait + 多格式导出 + 统计"),
          codeLine("├── tests/"),
          codeLine("│   └── integration_test.rs  # 集成测试"),
          codeLine("└── .gitignore"),
          emptyLine(),
          bodyPara("各模块之间的调用关系大概是：main.rs 调 cli.rs 解析命令，cli.rs 根据命令分别调 model、storage、search、export 来干活。model 定义了所有用到的数据结构，其他模块都依赖它。"),

          // 四、设计与实现
          h1("三、设计与实现"),
          bodyPara("整体设计思路比较简单——命令行工具嘛，就是「解析输入 → 找到对应功能 → 执行 → 输出结果」这个流程。下面说几个关键的设计决策："),
          emptyLine(),
          h2("3.1 命令行参数解析"),
          bodyPara("用的 clap 这个 crate，通过 derive 宏来定义命令和参数。Command 枚举把所有支持的命令都列出来了，包括 New、List、Search、Show、Edit、Delete、Tag、Untag、Category、Export、Stats 这些。clap 自动帮我们生成帮助信息和参数校验。"),
          emptyLine(),
          h2("3.2 数据存储方式"),
          bodyPara("笔记就存在本地文件系统的 notes/ 目录下，一个笔记一个 .md 文件。文件名就是笔记 id（由标题生成，转小写、空格换横线）。每个文件头部有 YAML front matter，格式如下："),
          emptyLine(),
          codeLine("---"),
          codeLine("title: \"笔记标题\""),
          codeLine("created: 2026-05-31"),
          codeLine("modified: 2026-05-31"),
          codeLine("tags: [rust, 学习]"),
          codeLine("category: 课堂笔记"),
          codeLine("---"),
          emptyLine(),
          bodyPara("这种格式的好处是 Markdown 编辑器也能直接认，编辑器不会把 YAML 头当代码渲染出来，有些编辑器甚至能识别 front matter 做筛选。storage 模块负责解析和生成这种格式。"),
          emptyLine(),
          h2("3.3 错误处理策略"),
          bodyPara("所有可能出错的操作都返回 Result<T, E>，用 ? 操作符往上抛。每个模块有自己的错误类型：StorageError、SearchError、ExportError，然后 cli 层统一 catch 住打印给用户。基本没有用 unwrap/expect（除了测试里）。"),

          // 五、各模块详细说明
          h1("四、各模块详细说明"),
          emptyLine(),
          h2("4.1 model.rs — 数据模型"),
          bodyPara("这个模块定义了项目里用到的所有核心数据结构和类型："),
          bodyPara("• Note 结构体：包含 id（字符串）、metadata（Metadata 类型）和 content（正文内容）。Note 实现了 Display trait，方便打印。"),
          bodyPara("• Metadata 结构体：记录笔记的元信息——标题、创建日期、修改日期、标签列表、分类。标签用的 HashSet<Tag> 存储，自动去重。"),
          bodyPara("• Tag 结构体：一个包装了 String 的新类型（newtype），实现了 PartialEq、Hash、Display 等trait。"),
          bodyPara("• Command 枚举：每个变体对应一条命令，比如 New { title }、List { tag, category, date, ... }、Search { keyword, regex } 等。List 和 Search 的筛选条件用 Option<String>，表示可选参数。"),
          bodyPara("• ExportFormat 枚举：Json / Html / Text，实现了 FromStr trait，这样从命令行字符串 \"json\" 就能解析成 ExportFormat::Json。"),
          bodyPara("这个模块体现的 Rust 特性：struct 定义、enum 定义、impl 块、trait 实现（Display、FromStr、PartialEq 等）、derive 宏、Option 类型。"),
          emptyLine(),
          h2("4.2 storage.rs — 文件存储"),
          bodyPara("这个模块管所有文件 I/O 操作，核心是 Storage 结构体："),
          bodyPara("• Storage 结构体：持有一个 PathBuf 表示笔记目录的路径。"),
          bodyPara("• create_note：新建笔记，自动生成 id 和 YAML 头，写入文件。如果文件已存在就返回 AlreadyExists 错误。"),
          bodyPara("• load_note / list_notes / delete_note：读取、列表、删除笔记。"),
          bodyPara("• edit_note：调用系统编辑器打开文件，编辑完更新修改日期。优先尝试 vscode，没有就用记事本。"),
          bodyPara("• parse_note_file：从 .md 文件内容解析出 Note 结构体。要处理有 front matter 和没有 front matter 两种情况，解析逻辑手动写的（没有用 yaml 库，因为格式比较简单固定）。"),
          bodyPara("错误处理方面，定义了 StorageError 枚举，包含 Io（系统IO错误）、NotFound、AlreadyExists、ParseError 等变体，实现了 From<std::io::Error> 方便用 ? 转换。"),
          bodyPara("这个模块体现的 Rust 特性：所有权（PathBuf 的移动语义）、Result 错误处理、From trait 实现、文件 I/O 操作、字符串解析。"),
          emptyLine(),
          h2("4.3 search.rs — 搜索模块"),
          bodyPara("搜索模块提供了好几种查找方式："),
          bodyPara("• filter_notes 泛型函数：接受一个笔记切片和一个闭包（Fn(&Note) -> bool），返回满足条件的笔记引用。这里用泛型约束 F: Fn(&Note) -> bool，所以传闭包、函数指针都行。"),
          bodyPara("• Filter 枚举：ByTag / ByCategory / ByDateRange / ByKeyword / ByRegex / All / Any。All 是「同时满足所有」，Any 是「满足任一」。filter_by_filter 函数递归地组合这些条件。"),
          bodyPara("• search_keyword：不区分大小写的关键词搜索，返回 SearchResult（包含笔记引用和匹配行号）。"),
          bodyPara("• search_regex：正则搜索，用 regex crate 编译正则表达式然后匹配。"),
          bodyPara("这个模块体现的 Rust 特性：泛型（filter_notes 的 F 参数）、生命周期（返回的引用和输入切片同生命周期）、闭包作为参数、enum 递归组合模式。"),
          emptyLine(),
          h2("4.4 export.rs — 导出模块"),
          bodyPara("导出模块的核心是 Exportable trait："),
          emptyLine(),
          codeLine("pub trait Exportable {"),
          codeLine("    fn export(&self, notes: &[Note]) -> Result<String, ExportError>;"),
          codeLine("    fn file_extension(&self) -> &str;"),
          codeLine("    fn export_to_file(&self, notes: &[Note], path: &Path) -> Result<(), ExportError> { ... }"),
          codeLine("}"),
          emptyLine(),
          bodyPara("三个实现：JsonExporter、HtmlExporter、TextExporter。其中 export_to_file 有默认实现（调用 export 拿到字符串再写文件），这样实现 trait 的类型只要写 export 方法就行。"),
          bodyPara("HtmlExporter 里有个 escape_html 函数来防 XSS，把 < > & \" 这些特殊字符转义掉。"),
          bodyPara("compute_stats 函数算统计信息——笔记数量、标签分布、分类分布，返回一个 NoteStats 结构体。"),
          bodyPara("还有一个 export_notes 泛型函数，接收 &dyn Exportable 做动态分发，演示了 trait object 的用法。"),
          bodyPara("这个模块体现的 Rust 特性：trait 定义与实现、trait 默认方法、dyn 动态分发、泛型函数、Result 错误处理。"),
          emptyLine(),
          h2("4.5 cli.rs — 命令行入口"),
          bodyPara("cli 模块就是把上面的模块串起来。run_command 函数接收一个 Command 枚举，match 分支到不同的处理逻辑。每个分支大概就是：调 storage 读写数据 → 调 search 过滤 → 调 export 输出 → 打印结果。"),
          bodyPara("命令行参数的解析用 clap 的 derive 模式，在 model.rs 里定义好 Command 枚举加上 clap 的 derive 宏，cli.rs 里调 clap 解析命令行输入，自动映射到 Command 变体。"),

          // 六、运行截图
          h1("五、运行截图"),
          bodyPara("（此处留空，后续补充运行截图）"),
          emptyLine(),
          emptyLine(),
          emptyLine(),

          // 七、遇到的问题与解决方法
          h1("六、遇到的问题与解决方法"),
          emptyLine(),
          h2("6.1 中文标题生成 id 的问题"),
          bodyPara("title_to_id 函数把标题转成文件名 id 的时候，一开始以为 is_alphanumeric() 只匹配英文字母和数字，结果中文字符也返回 true，所以中文标题生成的 id 保留了中文，比如 \"Rust 学习笔记\" 变成 \"rust-学习笔记\" 而不是 \"rust\"。后来想了想，中文保留在 id 里其实也合理（毕竟中文标题很常见），就改了测试期望来适配这个行为，而不是强行过滤掉中文。"),
          emptyLine(),
          h2("6.2 chrono 的 Datelike trait 需要显式导入"),
          bodyPara("用 NaiveDate 的 .year()、.month()、.day() 方法时，VSCode 的 Rust Analyzer 一直飘红报错，但 cargo build 却能过。查了一下才知道这些方法来自 chrono::Datelike 这个 trait，必须显式 use 了才能调用。编译器在某些情况下能自动找到，但 Rust Analyzer 更严格。加上 use chrono::Datelike 就好了。"),
          emptyLine(),
          h2("6.3 cargo clippy 的建议"),
          bodyPara("第一次跑 clippy 有 4 个 warning：io::Error::new(Other, msg) 建议换成 io::Error::other(msg)；map_or(false, ...) 建议换成 is_some_and(...)；filter_notes 函数的生命周期标注说可以省略；还有个闭包可以简化成方法引用。都是比较细节的写法优化，改完就 0 warning 了。"),
          emptyLine(),
          h2("6.4 项目路径含中文导致编译卡住"),
          bodyPara("一开始项目路径是 E:\\2026大三下课程任务\\2026RustFinalWork，cargo build 直接卡住不动了。后来把中文路径改成英文 E:\\2026DaSanXiaHomework\\2026RustFinalWork 就好了。Rust 工具链对中文路径支持不太好，算是踩了个坑。"),

          // 八、其他需要说明的内容
          h1("七、其他需要说明的内容"),
          h2("7.1 AI 使用情况"),
          bodyPara("本项目在开发过程中合理使用了 AI 辅助工具，主要用于以下几个方面："),
          bodyPara("• 项目结构规划：参考 AI 建议来划分模块（model/storage/search/export/cli），确定各模块的职责边界。"),
          bodyPara("• 代码实现参考：在部分模块的编写过程中，参考了 AI 生成的代码框架，然后根据项目实际需求进行了修改和调整。比如 Exportable trait 的设计、Filter 枚举的递归组合模式这些。"),
          bodyPara("• 问题排查：遇到 Datelike trait 飘红、clippy 警告这些问题时，参考了 AI 的分析来定位原因。"),
          bodyPara("• 文档编写：README 和本报告的初稿参考了 AI 的输出，然后做了去 AI 化的改写。"),
          bodyPara("总的来说 AI 在这个项目里更多是起辅助参考作用，核心设计和实现思路都是自己确定的，代码也逐行理解过、调试过。"),

          // 九、总结
          h1("八、总结"),
          bodyPara("这次 Rust 期末大作业让我对 Rust 的核心特性有了更深的理解。写完这个项目，感触比较深的有几点："),
          bodyPara("第一，所有权和借用确实在编译期就能拦住很多问题。比如搜索模块里返回笔记引用，编译器会强制你标注生命周期，确保引用不会比数据活得更久。一开始觉得烦，但习惯了之后反而觉得安全。"),
          bodyPara("第二，Result + ? 的错误处理模式写起来很舒服，不需要到处 try-catch，错误沿着调用链自动往上冒泡，到最外层统一处理就行。"),
          bodyPara("第三，trait 系统比想象中好用。Exportable trait 的默认方法、dyn 动态分发这些，写的时候很自然，不像 Java 的接口那么啰嗦。"),
          bodyPara("第四，测试在 Rust 里写起来确实方便。#[test] 加上 cargo test，单元测试和集成测试一跑就知道，46 个测试全绿的时候还是很有成就感的。"),
          bodyPara("不足的地方也有：项目的搜索功能还比较基础，只支持全文匹配和正则，没有做索引；导出的 HTML 也比较简陋，没有引入 CSS 样式。如果以后有时间可以继续改进。"),
        ]
      }
    ]
  });

  const buffer = await Packer.toBuffer(doc);
  const outputPath = "C:/Users/19923/Downloads/Rust期末作业报告.docx";
  fs.writeFileSync(outputPath, buffer);
  console.log("报告已生成: " + outputPath);
}

main().catch(err => { console.error(err); process.exit(1); });
