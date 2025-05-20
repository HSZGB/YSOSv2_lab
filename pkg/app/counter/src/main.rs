#![no_std]
#![no_main]

extern crate lib;
use lib::{sync::{Semaphore, SpinLock}, *};

const THREAD_COUNT: usize = 8;
static mut COUNTER_SEM: isize = 0;
static mut COUNTER_SPIN: isize = 0;

// 信号量方式
static SEM: Semaphore = Semaphore::new(0);
// 自旋锁方式
static SPIN: SpinLock = SpinLock::new();

fn main() -> isize {
    let pid = sys_fork();

    if pid == 0 {
        test_semaphore();
    } else {
        test_spin();
        sys_wait_pid(pid);
    }
    0
}

fn test_semaphore() {
    let mut pids = [0u16; THREAD_COUNT];
    SEM.init(1);

    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc_semaphore();
            sys_exit(0);
        } else {
            pids[i] = pid;
        }
    }

    let cpid = sys_get_pid();
    println!("[Semaphore] process #{} holds threads: {:?}", cpid, &pids);
    sys_stat(); // 保留系统调用
    for i in 0..THREAD_COUNT {
        println!("[Semaphore] #{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }
    println!("[Semaphore] COUNTER result: {}", unsafe { COUNTER_SEM });
}

fn test_spin() {
    let mut pids = [0u16; THREAD_COUNT];

    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc_spinlock();
            sys_exit(0);
        } else {
            pids[i] = pid;
        }
    }

    let cpid = sys_get_pid();
    println!("[SpinLock] process #{} holds threads: {:?}", cpid, &pids);
    sys_stat(); // 保留系统调用
    for i in 0..THREAD_COUNT {
        println!("[SpinLock] #{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }
    println!("[SpinLock] COUNTER result: {}", unsafe { COUNTER_SPIN });
}

fn do_counter_inc_semaphore() {
    for _ in 0..100 {
        SEM.wait();
        inc_counter_sem();
        SEM.signal();
    }
}

fn do_counter_inc_spinlock() {
    for _ in 0..100 {
        SPIN.acquire();
        inc_counter_spin();
        SPIN.release();
    }
}

fn inc_counter_sem() {
    unsafe {
        delay();
        let mut val = COUNTER_SEM;
        delay();
        val += 1;
        delay();
        COUNTER_SEM = val;
    }
}

fn inc_counter_spin() {
    unsafe {
        delay();
        let mut val = COUNTER_SPIN;
        delay();
        val += 1;
        delay();
        COUNTER_SPIN = val;
    }
}

#[inline(never)]
#[unsafe(no_mangle)]
fn delay() {
    for _ in 0..0x1000 {
        core::hint::spin_loop();
    }
}

entry!(main);