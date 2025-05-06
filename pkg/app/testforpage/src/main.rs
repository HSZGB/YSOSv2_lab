#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    let ptr : *mut u32 = 0xffffff0000000000 as *mut u32;
    unsafe { *ptr = 42; } // 应该触发 page fault
    0
}

entry!(main);