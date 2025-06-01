#![no_std]
#![no_main]

use lib::{vec::Vec, *};

extern crate lib;

fn main() -> isize {
    println!("YatSenOS Shell 启动");
    loop {
        // print!("shell> ");
        print!("\x1b[32m[>]\x1b[0m ");
        let input = stdin().read_line();
        let cmd = input.trim();
        let args: Vec<&str> = cmd.split_whitespace().collect();
        
        // match cmd {
        //     "exit" => sys_exit(0),
        //     "lsapp" => sys_list_app(),
        //     "ls" => sys_list_dir("/APP"),
        //     "ps" => sys_stat(),
        //     "clear" => print!("\x1b[2J\x1b[H"),
        //     "help" => show_help(),
        //     "" => continue, // 空命令不做任何处理
        //     // _ => println!("shell: command not found: {}", cmd),
        //     _ => {
        //         if cmd.starts_with("run ") {
        //             let prog = cmd.strip_prefix("run ").unwrap().trim();
        //             if prog.is_empty() {
        //                 println!("用法: run <程序名>");
        //             } else {
        //                 let pid = sys_spawn(prog);
        //                 if pid == 0 {
        //                     println!("shell: 无法启动程序: {}", prog);
        //                     continue;
        //                 }
        //                 sys_wait_pid(pid);
        //             }
        //         } else {
        //             println!("shell: command not found: {}", cmd);
        //         }
        //     }
        // }
        match args.as_slice() {
            ["exit"] => sys_exit(0),
            ["lsapp"] => sys_list_app(),
            ["ps"] => sys_stat(),
            ["clear"] => print!("\x1b[2J\x1b[H"),
            ["help"] => show_help(),
            ["ls"] => sys_list_dir("/"),
            ["ls", path] => sys_list_dir(path),
            ["cat", path] => {
                let fd = sys_open(path);
                let buf = &mut [0u8; 1024]; // 假设最大读取 1024 字节
                let size = sys_read(fd, buf);
                if fd == 0 {
                    println!("cat: {}: No such file or directory", path);
                } else {
                    let content = core::str::from_utf8(&buf[..size.unwrap()]).unwrap_or("无法解析内容");
                    println!("{}", content);
                }
                sys_close(fd);
            }
            ["run", prog] => {
                let pid = sys_spawn(prog);
                if pid == 0 {
                    println!("shell: 无法启动程序: {}", prog);
                } else {
                    sys_wait_pid(pid);
                }
            }
            [] => continue,
            _ => println!("shell: command not found: {}", cmd),
        }
    }
}

fn show_help() {
    println!("\x1b[33m============== YatSenOS Shell 帮助 ==============\x1b[0m");
    println!("作者: 黄镇邦 23342035");
    println!("\x1b[36m可用命令:\x1b[0m");
    println!("  exit         - 退出 shell");
    println!("  lsapp       - 列出所有可用应用程序");
    println!("  ls <path>   - 列出指定路径下的文件和目录");
    println!("  cat <file>  - 显示文件内容");
    println!("  ps           - 显示当前所有进程");
    println!("  run hello    - 运行 hello 程序");
    println!("  run fac      - 运行阶乘计算程序");
    println!("  run forktest - 运行 fork 测试程序");
    println!("  clear        - 清空屏幕");
    println!("  help         - 显示此帮助信息");
    println!("\x1b[33m================================================\x1b[0m");
}

entry!(main);