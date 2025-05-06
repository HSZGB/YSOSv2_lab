use crate::*;
use alloc::string::{String, ToString};
use alloc::vec;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    fn new() -> Self {
        Self
    }

    pub fn read_line(&self) -> String {
        // FIXME: allocate string
        // FIXME: read from input buffer
        //       - maybe char by char?
        // FIXME: handle backspace / enter...
        // FIXME: return string
        let mut buf = String::new();
        let mut char_buf = [0u8; 1];
        loop {
            if let Some(n) = sys_read(0, &mut char_buf) {
                if n == 0 {
                    continue;
                }
                let ch = char_buf[0] as char;
                match ch {
                    '\n' | '\r' => {
                        self::print!("\n");
                        break;
                    }
                    '\x08' | '\x7f' => {
                        if !buf.is_empty() {
                            self::print!("\x08\x20\x08");
                            buf.pop();
                        }
                    }
                    _ => {
                        buf.push(ch);
                        self::print!("{}", ch);
                    }
                }
            }
        }

        buf
    }
}

impl Stdout {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(1, s.as_bytes());
    }
}

impl Stderr {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(2, s.as_bytes());
    }
}

pub fn stdin() -> Stdin {
    Stdin::new()
}

pub fn stdout() -> Stdout {
    Stdout::new()
}

pub fn stderr() -> Stderr {
    Stderr::new()
}
