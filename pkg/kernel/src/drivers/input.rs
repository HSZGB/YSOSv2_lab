use alloc::string::String;
use crossbeam_queue::ArrayQueue;

type Key = u8;

lazy_static! {
    static ref INPUT_BUF: ArrayQueue<Key> = ArrayQueue::new(128);
}

#[inline]
pub fn push_key(key: Key) {
    if INPUT_BUF.push(key).is_err() {
        warn!("Input buffer is full. Dropping key '{:?}'", key);
    }
}

#[inline]
pub fn try_pop_key() -> Option<Key> {
    INPUT_BUF.pop()
}

pub fn pop_key() -> Key {
    loop {
        if let Some(key) = try_pop_key() {
            return key;
        }
    }
}

pub fn get_line() -> String {
    let mut line = String::with_capacity(128);
    loop {
        let c = pop_key();
        match c {
            b'\n' | b'\r' => { // 有些终端换行是'\r'
                print!("\n");
                break;
            },
            0x08 | 0x7F => {
                if !line.is_empty() {
                    line.pop();
                    crate::drivers::uart16550::backspace();
                }
            },
            _ => {
                line.push(c as char);
                print!("{}", c as char);
            }
        }
    }
    line
}