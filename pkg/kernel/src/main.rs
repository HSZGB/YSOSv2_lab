#![no_std]
#![no_main]

use log::info;
use storage::PartitionTable;
use ysos::*;
use ysos_kernel::{self as ysos, ata::AtaDrive};

extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    // AtaDrive::open(0, 0);
    let drive = AtaDrive::open(0, 0).unwrap();
    let mbr = storage::mbr::MbrTable::parse(drive).unwrap();
    let partitions = mbr.partitions().unwrap();
    info!("MBR Partitions: {:#?}", partitions);
    ysos::wait(spawn_init());
    ysos::shutdown();
}

pub fn spawn_init() -> proc::ProcessId {
    // NOTE: you may want to clear the screen before starting the shell
    // print!("\x1b[1;1H\x1b[2J");

    proc::list_app();
    proc::spawn("sh").unwrap()
    // proc::spawn("testforpage").unwrap()
    // proc::spawn("hello").unwrap()
    // loop{}
}
