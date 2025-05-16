#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    println!("YatSenOS Shell 启动");
    loop {
        // print!("shell> ");
        print!("\x1b[32m[>]\x1b[0m ");
        let input = stdin().read_line();
        let cmd = input.trim();
        
        match cmd {
            "exit" => sys_exit(0),
            "app" | "ls" => sys_list_app(),
            "ps" => sys_stat(),
            "hello" => {
                println!("启动 hello 程序...");
                let pid = sys_spawn("hello");
                println!("等待 hello 程序结束...");
                sys_wait_pid(pid);
            },
            "run hello" => {
                let pid = sys_spawn("hello");
                sys_wait_pid(pid);
            },
            "run fac" => {
                let pid = sys_spawn("fac");
                sys_wait_pid(pid);
            },
            "run forktest" => {
                let pid = sys_spawn("forktest");
                sys_wait_pid(pid);
            },
            "run sh" => {
                let pid = sys_spawn("sh");
                sys_wait_pid(pid);
            },
            "clear" => print!("\x1b[2J\x1b[H"),
            "help" => show_help(),
            "" => continue, // 空命令不做任何处理
            _ => println!("shell: command not found: {}", cmd),
        }
    }
}

fn show_help() {
    println!("\x1b[33m============== YatSenOS Shell 帮助 ==============\x1b[0m");
    println!("作者: 黄镇邦 23342035");
    println!("\x1b[36m可用命令:\x1b[0m");
    println!("  exit         - 退出 shell");
    println!("  app/ls       - 列出所有可用应用程序");
    println!("  ps           - 显示当前所有进程");
    println!("  hello        - 运行 hello 程序");
    println!("  run hello    - 运行 hello 程序");
    println!("  run fac      - 运行阶乘计算程序");
    println!("  run forktest - 运行 fork 测试程序");
    println!("  clear        - 清空屏幕");
    println!("  help         - 显示此帮助信息");
    println!("\x1b[33m================================================\x1b[0m");
}

entry!(main);