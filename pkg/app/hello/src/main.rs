#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    println!("Hello, world!!!");

    233
    // sys_exit(233)
}

entry!(main);
