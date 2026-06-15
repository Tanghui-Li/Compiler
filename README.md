# 编译原理大作业整理说明

本工作区现按两份正式大作业整理为顶层目录：

- `大作业1_词法和语法分析工具`：来自原 `E1/Assignment1`，包含大作业 1 的设计报告、Rust 源码、运行截图、展示文稿和最终提交包。
- `大作业2_语义分析与中间代码生成器`：来自原 `E1/Assignment2`，包含大作业 2 的设计报告、Rust 源码、运行截图、展示文稿和最终提交包。

顶层 `作业要求.txt` 来自原 `E1/要求`。原 `E2/要求` 内容相同，随旧副本保留在归档目录。

## 正式目录内容

每份作业目录保留以下核心内容：

- `(1)design.tex` 与 `(1)design.pdf`：设计报告源文件和 PDF。
- `(2)rust_parser` 或 `(2)rust_compiler`：Rust 工程源码和测试输入。
- `(3)screenshots`：运行截图。
- `(4)presentation.tex` 与 `(4)presentation.pdf`：展示文稿源文件和 PDF。
- `submission.zip`：最终提交压缩包。
- `【Rust版】...pptx`：课程原始要求或展示参考文件。

## 归档目录

`_archive` 保存整理时移出的内容，不作为正式提交入口：

- `_archive/E2_旧混合副本_未采用`：原 `E2`。该目录的报告和展示文稿标题仍是大作业 1，但代码目录像大作业 2，且源码时间早于正式大作业 2，因此仅保留为旧混合副本。
- `_archive/build_artifacts`：从源码工程中移出的 Rust `target` 构建产物。
- `_archive/latex_build_artifacts`：从报告和展示文稿目录中移出的 LaTeX 中间文件。
- `_archive/extracted_submission`：原先解压出来的重复 `submission` 目录；正式目录中已经保留对应 `submission.zip`。

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
