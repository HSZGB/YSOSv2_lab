use core::alloc::Layout;

use x86_64::VirtAddr;

use crate::proc;
use crate::proc::manager::get_process_manager;
use crate::proc::*;
use crate::utils::*;

use super::SyscallArgs;

pub fn spawn_process(args: &SyscallArgs) -> usize {
    // FIXME: get app name by args
    //       - core::str::from_utf8_unchecked
    //       - core::slice::from_raw_parts
    // FIXME: spawn the process by name
    // FIXME: handle spawn error, return 0 if failed
    // FIXME: return pid as usize

    let path = unsafe {
        core::str::from_utf8_unchecked(
            core::slice::from_raw_parts(args.arg0 as *const u8, args.arg1 as usize),
        )
    };

    // match proc::spawn(app_name) {
    //     Some(pid) => pid.0 as usize,  // 成功启动进程，返回进程 ID
    //     None => 0,  // 进程启动失败，返回 0 表示失败
    // }
    match proc::fs_spawn(path) {
        Some(pid) => pid.0 as usize,  // 成功启动进程，返回进程 ID
        None => 0,  // 进程启动失败，返回 0 表示失败
    }
}

pub fn sys_write(args: &SyscallArgs) -> usize {
    // FIXME: get buffer and fd by args
    //       - core::slice::from_raw_parts
    // FIXME: call proc::write -> isize
    // FIXME: return the result as usize

    let fd = args.arg0 as u8;
    let buf = unsafe { core::slice::from_raw_parts(args.arg1 as *const u8, args.arg2) };

    let written = proc::write(fd, buf);
    written as usize
    // 0
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    // FIXME: just like sys_write
    let fd = args.arg0 as u8;
    let buf = unsafe {
        core::slice::from_raw_parts_mut(args.arg1 as *mut u8, args.arg2)
    };

    let read = proc::read(fd, buf);
    read as usize
}

pub fn get_pid() -> u16 {
    get_process_manager().current().pid().0 as u16
}

pub fn sys_fork(context: &mut ProcessContext) {
    proc::fork(context);
}

pub fn sys_wait_pid(args: &SyscallArgs, context: &mut ProcessContext) {
    let pid = ProcessId(args.arg0 as u16);
    proc::wait_pid(pid, context);
}

pub fn sys_sem(args: &SyscallArgs, context: &mut ProcessContext) {
    match args.arg0 {
        0 => context.set_rax(new_sem(args.arg1 as u32, args.arg2)),
        1 => context.set_rax(remove_sem(args.arg1 as u32)),
        2 => sem_signal(args.arg1 as u32, context),
        3 => sem_wait(args.arg1 as u32, context),
        _ => context.set_rax(usize::MAX),
    }
}

pub fn exit_process(args: &SyscallArgs, context: &mut ProcessContext) {
    // FIXME: exit process with retcode
    proc::exit(args.arg0 as isize, context)
}

pub fn list_process() {
    // FIXME: list all processes
    proc::print_process_list();
}

pub fn sys_allocate(args: &SyscallArgs) -> usize {
    let layout = unsafe { (args.arg0 as *const Layout).as_ref().unwrap() };

    if layout.size() == 0 {
        return 0;
    }

    let ret = crate::memory::user::USER_ALLOCATOR
        .lock()
        .allocate_first_fit(*layout);

    match ret {
        Ok(ptr) => ptr.as_ptr() as usize,
        Err(_) => 0,
    }
}

pub fn sys_deallocate(args: &SyscallArgs) {
    let layout = unsafe { (args.arg1 as *const Layout).as_ref().unwrap() };

    if args.arg0 == 0 || layout.size() == 0 {
        return;
    }

    let ptr = args.arg0 as *mut u8;

    unsafe {
        crate::memory::user::USER_ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), *layout);
    }
}

pub fn list_dir(args: &SyscallArgs) {
    let root = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            args.arg0 as *const u8,
            args.arg1,
        ))
    };
    crate::drivers::filesystem::ls(root);
}

pub fn sys_open(args: &SyscallArgs) -> usize {
    let path = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            args.arg0 as *const u8,
            args.arg1,
        ))
    };

    match open(path) {
        Some(fd) => fd as usize,  // 成功打开文件，返回文件描述符
        None => 0,  // 打开文件失败，返回 0
    }

}

pub fn sys_close(args: &SyscallArgs) -> usize {
    let fd = args.arg0 as u8;

    if crate::proc::close(fd) {
        1  // 成功关闭文件，返回 1
    } else {
        0  // 关闭文件失败，返回 0
    }
}

pub fn sys_brk(args: &SyscallArgs) -> usize {
    let new_heap_end = if args.arg0 == 0 {
        None
    } else {
        Some(VirtAddr::new(args.arg0 as u64))
    };
    match brk(new_heap_end) {
        Some(new_heap_end) => new_heap_end.as_u64() as usize,
        None => !0,
    }
}