#![no_std]
#![no_main]

use lib::{boxed::Box, string::{String, ToString}, vec::Vec, *};

extern crate lib;

static mut CURRENT_DIR: &str = "/";

fn normalize_path(path: &str, current_dir: &str) -> String {
    // 如果是绝对路径，直接处理
    let base_path = if path.starts_with("/") {
        "".to_string()
    } else {
        current_dir.to_string()
    };
    
    // 分割路径
    let mut parts: Vec<&str> = if path.starts_with("/") {
        path.split('/').filter(|s| !s.is_empty()).collect()
    } else {
        let mut base_parts: Vec<&str> = base_path.split('/').filter(|s| !s.is_empty()).collect();
        let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        base_parts.extend(path_parts);
        base_parts
    };
    
    // 处理 . 和 .. 
    let mut result_parts = Vec::new();
    
    for part in parts {
        match part {
            "." => {
                // 当前目录，跳过
                continue;
            }
            ".." => {
                // 父目录，删除上一个部分
                if !result_parts.is_empty() {
                    result_parts.pop();
                }
            }
            _ => {
                result_parts.push(part);
            }
        }
    }
    
    // 构建结果路径
    if result_parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", result_parts.join("/"))
    }
}

fn main() -> isize {
    println!("YatSenOS Shell 启动");
    loop {
        // print!("shell> ");
        print!("\x1b[32m{}[>]\x1b[0m ", unsafe { CURRENT_DIR });
        let input = stdin().read_line();
        let cmd = input.trim();
        let args: Vec<&str> = cmd.split_whitespace().collect();

        match args.as_slice() {
            ["exit"] => sys_exit(0),
            ["lsapp"] => sys_list_app(),
            ["ps"] => sys_stat(),
            ["clear"] => print!("\x1b[2J\x1b[H"),
            ["help"] => show_help(),
            ["ls"] => sys_list_dir(unsafe{CURRENT_DIR}),
            ["ls", path] => sys_list_dir(&normalize_path(path, unsafe { CURRENT_DIR })),
            ["cat"] => {
                println!("cat: 请指定文件路径");
            }
            ["cat", path] => {
                let fd = sys_open(path);
                let buf = &mut [0u8; 1024]; // 假设最大读取 1024 字节
                let size = sys_read(fd, buf);
                if fd == 0 {
                    println!("cat: {}: No such file or directory", path);
                } else {
                    let content = core::str::from_utf8(&buf[..size.unwrap()]).unwrap_or("无法解析内容");
                    println!("{}", content);
                    sys_close(fd);
                }
            }
            ["cd", path] => {
                let normalized_path = unsafe { normalize_path(path, CURRENT_DIR) };
                unsafe {
                    CURRENT_DIR = Box::leak(normalized_path.into_boxed_str());
                }
            }
            ["run", path] => {
                let normalized_path = unsafe { normalize_path(path, CURRENT_DIR) };

                
                let pid = sys_spawn(&normalized_path);
                if pid == 0 {
                    println!("shell: 无法启动程序: {}", normalized_path);
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