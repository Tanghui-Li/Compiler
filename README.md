# 编译原理大作业工作区说明

本工作区已按两份正式大作业整理，并额外准备了一份合并汇报稿：

- `Presentation_Group8.tex` / `Presentation_Group8.pdf`：第 8 组合并汇报 Beamer，主题为 Berlin，覆盖大作业 1 和大作业 2。
- `Presentation_Group8_逐字稿.md`：配套 8 分钟逐字稿，按冉子易、高胜寒、李堂辉三人分段。
- `大作业1_词法和语法分析工具`：词法分析、递归下降语法分析和 AST 构造。
- `大作业2_语义分析与中间代码生成器`：在 AST 基础上做语义检查，并生成四元式中间代码。
- `作业要求.txt`：课程给出的两次大作业要求。
- `_archive`：整理时移出的旧副本、构建产物、LaTeX 中间文件和重复解压目录，不作为正式入口。

## 汇报稿说明

最终汇报文件为：

```text
Presentation_Group8.pdf
```

对应源码为：

```text
Presentation_Group8.tex
```

汇报按 8 分钟口头展示重新收紧，重点放在本实现的连续前端流程：

1. 大作业 1 把类 Rust 源程序结构化为 Token 和 AST。
2. AST 保留了类型、左值、引用、数组、元组、表达式块和控制流等信息。
3. 大作业 2 继续消费 AST，通过符号表栈、返回类型状态、循环栈和错误列表完成语义诊断。
4. 合法程序被线性化为四元式 IR，其中表达式由临时变量承接，分支和循环由标签与跳转承接。

## 两份作业的实现重点

### 大作业 1：词法和语法分析工具

正式目录：

```text
大作业1_词法和语法分析工具
```

核心内容：

- `(1)design.tex` / `(1)design.pdf`：设计报告。
- `(2)rust_parser`：Rust 源码工程。
- `(3)screenshots`：运行截图。
- `(4)presentation.tex` / `(4)presentation.pdf`：原单项展示稿。
- `submission.zip`：最终提交压缩包。

关键源码：

- `src/token.rs`：Token 类型、显示格式和类别划分。
- `src/lexer.rs`：游标式扫描器，处理关键字、标识符、数字、注释和双字符符号。
- `src/parser.rs`：手写递归下降 Parser。
- `src/ast.rs`：AST 节点定义和树形打印。

实现特色：

- 语法覆盖 `&T`、`&mut T`、数组、元组、区间、表达式块、`if` 表达式、`while/for/loop` 和 `break/continue`。
- Parser 通过函数层次表达优先级和结构层次，便于定位错误和继续扩展。
- AST 是两次作业的衔接契约，后续语义分析和 IR 生成直接消费这套结构。

### 大作业 2：语义分析与中间代码生成器

正式目录：

```text
大作业2_语义分析与中间代码生成器
```

核心内容：

- `(1)design.tex` / `(1)design.pdf`：设计报告。
- `(2)rust_compiler`：Rust 源码工程。
- `(3)screenshots`：运行截图。
- `(4)presentation.tex` / `(4)presentation.pdf`：原单项展示稿。
- `submission.zip`：最终提交压缩包。

关键源码：

- `src/compiler.rs`：语义检查和 IR 生成主逻辑。
- `src/ir.rs`：`Op`、`Arg`、`Quadruple` 和 `IRProgram`。
- `src/ast.rs` / `src/parser.rs` / `src/lexer.rs`：沿用前端结构。

实现特色：

- `scopes: Vec<HashMap<String, SymbolInfo>>` 管理嵌套作用域，并记录变量类型、可变性和 IR 参数。
- `current_return_type` 检查返回语句和函数签名是否匹配。
- `loop_stack` 同时用于检查 `break/continue` 合法性和生成循环跳转目标。
- `errors: Vec<String>` 汇总语义错误，避免只报告第一个错误。
- 四元式 IR 用 `tN` 表示临时变量，用 `LN` 表示控制流标签。

## 运行源码

大作业 1：

```bash
cd "大作业1_词法和语法分析工具/(2)rust_parser"
cargo run --release
```

大作业 2：

```bash
cd "大作业2_语义分析与中间代码生成器/(2)rust_compiler"
cargo run --release
```

## 验证状态

已检查两个 Rust 工程的可编译性：

```bash
cargo check
```

整理时两个正式工程均无编译错误；大作业 2 会有若干未使用变量或未使用字段的 warning，不影响运行。

## 归档目录

`_archive` 仅用于保留历史材料和整理时移出的文件：

- `_archive/E2_旧混合副本_未采用`：原 `E2` 旧混合副本。
- `_archive/build_artifacts`：Rust `target` 构建产物。
- `_archive/latex_build_artifacts`：LaTeX 编译中间文件。
- `_archive/extracted_submission`：重复解压的提交包内容。
