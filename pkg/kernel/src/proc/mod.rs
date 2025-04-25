mod context;
mod data;
pub mod manager;
mod paging;
mod pid;
mod process;
mod processor;

mod vm;


use manager::*;
use process::*;
use processor::get_pid;
use vm::ProcessVm;
use x86::current;
use crate::memory::PAGE_SIZE;

use alloc::string::String;
pub use context::ProcessContext;
pub use paging::PageTableContext;
pub use data::ProcessData;
pub use pid::ProcessId;

use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::VirtAddr;
pub const KERNEL_PID: ProcessId = ProcessId(1);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Running,
    Ready,
    Blocked,
    Dead,
}

/// init process manager
pub fn init() {
    let proc_vm = ProcessVm::new(PageTableContext::new()).init_kernel_vm();

    trace!("Init kernel vm: {:#?}", proc_vm);

    // kernel process
    let kproc = Process::new(
        String::from("kernel"),
        None,
        Some(proc_vm),
        None, // ???
    );
    manager::init(kproc);

    info!("Process Manager Initialized.");
}

pub fn switch(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // 如果当前的在Running，改变状态，扔进队列
        // 如果似了或阻塞了，不扔进ready_queue
        
        // FIXME: switch to the next process
        //      - save current process's context && update status
        let manager = get_process_manager();
        let proc = manager.current();

        if proc.read().status() == ProgramStatus::Running {
            manager.save_current(context); // 这里会改变成ready
            //      - handle ready queue update
            manager.push_ready(get_pid());
        }

        //      - restore next process's context
        manager.switch_next(context);
        // debug!("hello");
    });
}

pub fn spawn_kernel_thread(entry: fn() -> !, name: String, data: Option<ProcessData>) -> ProcessId {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let entry = VirtAddr::new(entry as usize as u64);
        get_process_manager().spawn_kernel_thread(entry, name, data)
    })
}

pub fn print_process_list() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().print_process_list();
    })
}

pub fn env(key: &str) -> Option<String> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: get current process's environment variable
        get_process_manager().current().read().env(key)
    })
}

pub fn process_exit(ret: isize) -> ! {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().kill_current(ret);
    });

    loop {
        x86_64::instructions::hlt();
    }
}

pub fn handle_page_fault(addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().handle_page_fault(addr, err_code)
    })
}
