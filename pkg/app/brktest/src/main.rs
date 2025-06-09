#![no_std]
#![no_main]

use lib::*;

extern crate lib;

const HEAP_SIZE: usize = 8 * 1024 - 8; // 8 KiB

fn main() -> isize {

    let heap_start = sys_brk(None).unwrap();
    let heap_end = heap_start + HEAP_SIZE;

    let ret = sys_brk(Some(heap_end)).expect("Failed to allocate heap");

    assert!(ret == heap_end, "Failed to allocate heap");

    println!("heap_start = {:#x}, heap_end = {:#x}, ret = {:#x}", heap_start, heap_end, ret);
    println!("Heap allocated successfully, ret = {:#x}", ret);
    // heap_start = 0x200000000000, heap_end = 0x200000001ff8, ret = 0x200000001ff8

    let ptr = heap_start as *mut u8;

    // 写入测试
    unsafe {
        *ptr = 42;
        assert_eq!(*ptr, 42);
    }

    // 大块写入/读取测试
    unsafe {
        for i in 0..HEAP_SIZE {
            *ptr.add(i) = (i % 256) as u8;
        }
        for i in 0..HEAP_SIZE {
            assert_eq!(*ptr.add(i), (i % 256) as u8);
        }
    }

    let ret = sys_brk(Some(heap_start)).expect("Failed to deallocate heap");
    assert!(ret == heap_start, "Failed to deallocate heap");
    println!("Heap deallocated successfully, ret = {:#x}", ret);

    233
}

entry!(main);