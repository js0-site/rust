use add_space::add_space;
use aok::{OK, Void};

#[static_init::constructor(0)]
extern "C" fn _loginit() {
  log_init::init();
}

#[test]
fn test() -> Void {
  for (txt, exp) in [
    (r#"State::Letter"#, r#"State::Letter"#),
    (
      "OAuth 2.0鉴权用户只能查询到通过OAuth 2.0鉴权创建的会议",
      "OAuth 2.0 鉴权用户只能查询到通过 OAuth 2.0 鉴权创建的会议",
    ),
    ("/* block comment */", "/* block comment */"),
    ("/* block 注释 */", "/* block 注释 */"),
    ("// line comment", "// line comment"),
    ("// line 注释", "// line 注释"),
    // from: test_end_space
    ("a ", "a "),
    ("a  ", "a  "),
    ("abc啊 ", "abc 啊 "),
    // from: test_newline_or_tab
    ("a\nb", "a\nb"),
    ("a\rb", "a\rb"),
    ("a\tb", "a\tb"),
    ("中\nb", "中\nb"),
    ("中\rb", "中\rb"),
    ("中\tb", "中\tb"),
    // from: test_spacing
    ("中文English", "中文 English"),
    ("中文English中文", "中文 English 中文"),
    ("中文123", "中文 123"),
    ("123中文", "123 中文"),
    ("中文!", "中文!"),
    ("中文?", "中文?"),
    ("价格是$50和¥300", "价格是 $50 和 ¥300"),
    ("价格是¥300", "价格是 ¥300"),
    (
      "当你凝视着bug，bug也凝视着你",
      "当你凝视着 bug，bug 也凝视着你",
    ),
    (
      "与PM战斗的人，应当小心自己不要成为PM",
      "与 PM 战斗的人，应当小心自己不要成为 PM",
    ),
    ("中文和拉丁字母English混排", "中文和拉丁字母 English 混排"),
    (
      "中文数字１２３４５６７８９０和半角数字1234567890混排",
      "中文数字１２３４５６７８９０和半角数字 1234567890 混排",
    ),
    (
      "使用了Python的print()函数打印\"你好,世界\"",
      "使用了 Python 的 print() 函数打印\"你好,世界\"",
    ),
    (
      "价格人民币¥100美元$100欧元€100英镑£100",
      "价格人民币 ¥100 美元 $100 欧元 €100 英镑 £100",
    ),
    ("全角空格　和半角空格 混用", "全角空格　和半角空格 混用"),
    (
      "AＡBＢCＣ和abc以及１２３和123混排",
      "AＡBＢCＣ 和 abc 以及１２３和 123 混排",
    ),
    ("文件保存在~/Documents目录", "文件保存在 ~/Documents 目录"),
    // from: test_symbols
    (
      "用户目录是~，完整路径是~/Documents",
      "用户目录是 ~，完整路径是 ~/Documents",
    ),
    ("函数add(a,b)返回a+b", "函数 add(a,b) 返回 a+b"),
    (
      "文件保存在/usr/local/bin/目录",
      "文件保存在/usr/local/bin/目录",
    ),
    (
      "网址是example.com而不是example。com",
      "网址是 example.com 而不是 example。com",
    ),
    (r#"他说"这很好"然后离开了"#, r#"他说"这很好"然后离开了"#),
    (
      "安装命令是npm install --save-dev @types/react使用v16.8版本",
      "安装命令是 npm install --save-dev @types/react 使用 v16.8 版本",
    ),
    // ("价格是$50和¥300", "价格是 $50 和 ¥300"), // 重复
    (
      "name|age|gender表示不同字段",
      "name|age|gender 表示不同字段",
    ),
    (
      "5+3*2=11，需要满足x>0且y<100",
      "5+3*2=11，需要满足 x>0 且 y<100",
    ),
    (
      "命令是`ls -la`，注意不要用''",
      "命令是 `ls -la`，注意不要用''",
    ),
    (r#"\t"#, r#"\t"#),
    // from: test_string_with_escapes
    ("你好\n world\t!", "你好\n world\t!"),
    (r#"你好\n world\t!"#, r#"你好\n world\t!"#),
    ("你好world", "你好 world"),
    (
      "请参阅我们的[贡献指南](CONTRIBUTING.md)，了解如何上手的详细信息。",
      "请参阅我们的[贡献指南](CONTRIBUTING.md)，了解如何上手的详细信息。",
    ),
    (
      r#"测试<img alt="图片描述">一下"#,
      r#"测试<img alt="图片描述">一下"#,
    ),
    (
      r#"翻译能够完美保持Markdown的格式。"#,
      r#"翻译能够完美保持 Markdown 的格式。"#,
    ),
    (
      r#"翻译能够完美保持`Markdown`的格式。"#,
      r#"翻译能够完美保持 `Markdown` 的格式。"#,
    ),
    (r#"第N次"#, r#"第N次"#),
    (r#"#!/usr/bin/env bun"#, r#"#!/usr/bin/env bun"#),
    ("中文\"hello\"中文", "中文\"hello\"中文"),
    ("中文'hello'中文", "中文'hello'中文"),
    ("中文\"hello world\"中文", "中文\"hello world\"中文"),
    ("中文'hello world'中文", "中文'hello world'中文"),
    ("中文\"hello\"中文\"world\"", "中文\"hello\"中文\"world\""),
    ("中文'hello'中文'world'", "中文'hello'中文'world'"),
    (r#"中文"hello\"world"中文"#, r#"中文"hello\"world"中文"#),
    (r#"中文'hello\'world'中文"#, r#"中文'hello\'world'中文"#),
    ("测试\"hello中文\"测试", "测试\"hello中文\"测试"),
    ("测试'hello中文'测试", "测试'hello中文'测试"),
    (
      "./dev.js ./ui/webc.site/组件名",
      "./dev.js ./ui/webc.site/组件名",
    ),
    (
      "`./dev.js ./ui/webc.site/组件名`",
      "`./dev.js ./ui/webc.site/组件名`",
    ),
    (
      "`./dev.js ./ui/webc.site/组件名`保存",
      "`./dev.js ./ui/webc.site/组件名`保存",
    ),
    (
      "现在  `./dev.js ./ui/webc.site/组件名` 保存会变成  `./dev.js ./ui/webc.site/组件名 ` ，`前面多了一个空格",
      "现在  `./dev.js ./ui/webc.site/组件名` 保存会变成  `./dev.js ./ui/webc.site/组件名 ` ，`前面多了一个空格",
    ),
    ("a.b", "a.b"),
    ("a.B", "a.B"),
    ("a.Bc", "a.Bc"),
    ("A.B", "A.B"),
    ("A.Bc", "A.Bc"),
    ("v1.0", "v1.0"),
    ("1.2", "1.2"),
    ("Hello.World", "Hello.World"),
    ("CONTRIBUTING.MD", "CONTRIBUTING.MD"),
    ("```txt\n测试test一下\n```", "```txt\n测试test一下\n```"),
    ("```\n测试test一下\n```", "```\n测试test一下\n```"),
    (
      "```rust\nlet a = `测试test一下`;\n```",
      "```rust\nlet a = `测试test一下`;\n```",
    ),
    (
      "【地球联合政府】《灵舟计划远征协议》",
      "【地球联合政府】《灵舟计划远征协议》",
    ),
    (
      "地球联合政府【地球联合政府】《灵舟计划远征协议》协议",
      "地球联合政府【地球联合政府】《灵舟计划远征协议》协议",
    ),
    (
      "【地球联合政府】《灵舟计划远征协议》test协议",
      "【地球联合政府】《灵舟计划远征协议》test 协议",
    ),
    (
      "【地球联合政府】《灵舟计划远征协议》test",
      "【地球联合政府】《灵舟计划远征协议》test",
    ),
    (
      "test【地球联合政府】《灵舟计划远征协议》",
      "test【地球联合政府】《灵舟计划远征协议》",
    ),
    (
      "1999 年 5月 12 日",
      "1999年5月12日",
    ),
    (
      "2026年 7月 29日",
      "2026年7月29日",
    ),
    (
      "1999年5月12日",
      "1999年5月12日",
    ),
    (
      ">　测试",
      ">　测试",
    ),
    (
      ">　",
      ">　",
    ),
    (
      "全角空格　>不增加半角空格",
      "全角空格　>不增加半角空格",
    ),
  ] {
    let add = add_space(txt);
    if add != exp {
      println!("\nFAILED CASE:\n  input:    {txt:?}\n  actual:   {add:?}\n  expected: {exp:?}");
      assert_eq!(add, exp);
    }
  }

  OK
}
