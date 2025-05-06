use core::alloc::Layout;

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

    let app_name = unsafe {
        core::str::from_utf8_unchecked(
            core::slice::from_raw_parts(args.arg0 as *const u8, args.arg1 as usize),
        )
    };

    match proc::spawn(app_name) {
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

pub fn sys_wait_pid(args: &SyscallArgs) -> isize {
    let pid = ProcessId(args.arg0 as u16);

    if let Some(code) = get_process_manager().get_exit_code(pid) {
        code // 已退出，返回退出码
    } else {
        23333   // 未退出，返回约定值（例如 23333）
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
