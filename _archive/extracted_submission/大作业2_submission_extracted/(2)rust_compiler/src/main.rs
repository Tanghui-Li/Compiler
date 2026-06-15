mod token;
mod lexer;
mod ast;
mod parser;
mod ir;
mod compiler;

use std::env;
use std::fs;
use lexer::Lexer;
use parser::Parser;
use ast::print_ast;

fn main() {
    let args: Vec<String> = env::args().collect();

    let source = if args.len() > 1 {
        // Read from file
        match fs::read_to_string(&args[1]) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("错误: 无法读取文件 '{}': {}", args[1], e);
                std::process::exit(1);
            }
        }
    } else {
        // Use built-in test cases
        get_test_source()
    };

    println!("========== 源代码 ==========");
    println!("{}", source);

    // ---- Lexical Analysis ----
    println!("\n========== 词法分析结果 ==========");
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("词法分析错误: {}", e);
            std::process::exit(1);
        }
    };

    for tok in &tokens {
        println!("  {}", tok);
    }

    // ---- Syntax Analysis ----
    println!("\n========== 语法分析结果 ==========");
    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(program) => {
            print_ast(&program, 0);
            println!("\n语法分析成功！");
            
            println!("\n========== 语义分析与中间代码 ==========");
            let mut compiler = compiler::Compiler::new();
            compiler.compile_program(&program);
            
            if compiler.has_errors() {
                compiler.print_errors();
                println!("\n语义分析失败，包含静态语义错误！");
            } else {
                println!("语义分析通过，生成四元式序列：\n");
                compiler.get_ir().print();
            }
        }
        Err(e) => {
            eprintln!("语法分析错误: {}", e);
            std::process::exit(1);
        }
    }
}

fn get_test_source() -> String {
    r#"
// ===== 1.1 基础程序 =====
fn program_1_1() {
}

// ===== 1.2 语句 =====
fn program_1_2() {
    ;;;;;;
}

// ===== 1.3 返回语句 =====
fn program_1_3() {
    return ;
}

// ===== 1.4 函数输入 =====
fn program_1_4(mut a:i32) {
}

// ===== 1.5 函数输出 =====
fn program_1_5() -> i32 {
    return 1;
}

// ===== 2.1 变量声明语句 =====
fn program_2_1() {
    let mut a;
    let mut b:i32;
}

// ===== 2.2 赋值语句 =====
fn program_2_2(mut a:i32) {
    a=32;
}

// ===== 2.3 变量声明赋值语句 =====
fn program_2_3() {
    let mut a=1;
    let mut b:i32=1;
}

// ===== 3.1 基本表达式 =====
fn program_3_1__1() {
    0;
    (1);
    ((2));
    (((3)));
}

fn program_3_1__2(mut a:i32) {
    a;
    (a);
    ((a));
    (((a)));
}

// ===== 3.2 比较运算 =====
fn program_3_2() {
    1<2;
    3>4;
}

// ===== 3.3 加减运算 =====
fn program_3_3() {
    1+2;
    3-4;
}

// ===== 3.4 乘除运算 =====
fn program_3_4() {
    1*2;
    3/4;
}

// ===== 3.5 函数调用 =====
fn program_3_5__1() {
}

fn program_3_5__2() {
    program_3_5__1();
}

// ===== 4.1 选择结构 if =====
fn program_4_1(a:i32) -> i32 {
    if a>0 {
        return 1;
    }
    return 0;
}

// ===== 4.2 选择结构 if-else =====
fn program_4_2(a:i32) -> i32 {
    if a>0 {
        return 1;
    } else {
        return 0;
    }
}

// ===== 4.3 选择结构 if-else if-else =====
fn program_4_3(a:i32) -> i32 {
    if a>0 {
        return a+1;
    } else if a<0 {
        return a-1;
    } else {
        return 0;
    }
}

// ===== 5.1 while循环 =====
fn program_5_1(mut n:i32) {
    while n>0 {
        n=n-1;
    }
}

// ===== 5.2 for循环 =====
fn program_5_2(mut n:i32) {
    for mut i in 1..n+1 {
        n=n-1;
    }
}

// ===== 5.3 loop循环 =====
fn program_5_3() {
    loop {
    }
}

// ===== 5.4 break 和 continue =====
fn program_5_4() {
    while 1==0 { continue; }
    while 1==1 { break; }
}

// ===== 6.1 变量不可变属性 =====
fn program_6_1() {
    let a:i32;
    let b;
    let c:i32=1;
    let d=2;
}

// ===== 6.2 不可变引用 =====
fn program_6_2(a:i32) {
    let b:&i32=&a;
}

// ===== 6.3 可变引用 =====
fn program_6_3(mut a:i32) {
    let mut b:&mut i32=&mut a;
}

// ===== 6.4 借用 (解引用) =====
fn program_6_4(a:&mut i32) {
    let b=*a;
    *a=3;
}

// ===== 7.2 函数表达式块作为函数体 =====
fn program_7_2(mut x:i32,mut y:i32) -> i32 {
    let mut t=x*x+x;
    t=t+x*y;
    return t;
}

// ===== 8.1 数组类型 =====
fn program_8_1() {
    let mut a:[i32;3];
}

// ===== 8.2 数组表达式 =====
fn program_8_2(mut a:[i32;3]) {
    a=[1,2,3];
}

// ===== 8.3 数组元素 =====
fn program_8_3(mut a:[i32;3]) {
    let mut b:i32=a[0];
    a[0]=1;
}

// ===== 9.1 元组类型 =====
fn program_9_1() {
    let a:(i32,i32);
}

// ===== 9.2 元组表达式 =====
fn program_9_2(mut a:(i32,i32)) {
    a=(1,2);
}

// ===== 9.3 元组元素 =====
fn program_9_3(mut a:(i32,i32)) {
    let mut b:i32=a.0;
    a.0=1;
}
#
"#.to_string()
}
