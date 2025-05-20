#![no_std]
#![no_main]

extern crate lib;
use lib::{sync::Semaphore, *};

const NPROC: usize = 16;          // 总进程数
const NMSG: usize = 10;           // 每个生产者/消费者消息数
const QUEUE_CAP: usize = 8;       // 队列容量，可尝试 1/4/8/16

static SEM_MUTEX: Semaphore = Semaphore::new(0x1001); // 互斥信号量
static SEM_FULL: Semaphore = Semaphore::new(0x1002);  // 队列中已有消息数
static SEM_EMPTY: Semaphore = Semaphore::new(0x1003); // 队列剩余空位数
static mut COUNTER: isize = 0;                          // 用于模拟消息队列的元素计数

fn main() -> isize {
    SEM_MUTEX.init(1);
    SEM_FULL.init(0);
    SEM_EMPTY.init(QUEUE_CAP);

    println!("Parent: semaphores created, queue cap={}", QUEUE_CAP);

    let mut pids = [0u16; NPROC];

    for i in 0..NPROC {
        let pid = sys_fork();
        if pid == 0 {
            if i < NPROC / 2 {
                producer(i as u16);
            } else {
                consumer(i as u16);
            }
            sys_exit(0);
        } else {
            pids[i] = pid;
        }
    }

    delay();
    sys_stat();
    // 这里可能还没进行信号量的wait就已经查询

    for &pid in pids.iter() {
        println!("Parent waiting child {}...", pid);
        sys_wait_pid(pid);
    }

    let left = unsafe { COUNTER };
    println!("All children exited. COUNTER (queue) size: {}", left);
    left
}

fn producer(idx: u16) {
    for n in 0..NMSG {
        SEM_EMPTY.wait();
        SEM_MUTEX.wait();

        unsafe {
            COUNTER += 1;
        }
        println!("[Producer {}] produced: msg {}, queue count: {}", idx, n, unsafe{COUNTER});

        SEM_MUTEX.signal();
        SEM_FULL.signal();
    }
}

fn consumer(idx: u16) {
    for n in 0..NMSG {
        SEM_FULL.wait();
        SEM_MUTEX.wait();

        unsafe {
            COUNTER -= 1;
        }
        println!("[Consumer {}] consumed: msg {}, queue count: {}", idx, n, unsafe{COUNTER});

        SEM_MUTEX.signal();
        SEM_EMPTY.signal();
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