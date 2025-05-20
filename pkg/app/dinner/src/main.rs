#![no_std]
#![no_main]

extern crate lib;
use lib::*;

const N: usize = 5;

// 5根筷子的信号量
static CHOPSTICK: [Semaphore; N] = semaphore_array![0, 1, 2, 3, 4];
// 服务生信号量，最多允许N-1个哲学家同时尝试拿筷子，防止死锁
static WAITER: Semaphore = Semaphore::new(100);

fn main() -> isize {
    // 初始化信号量
    for stick in &CHOPSTICK {
        stick.init(1);
    }
    WAITER.init(N - 1);

    let mut pids = [0u16; N];
    for i in 0..N {
        let pid = sys_fork();
        if pid == 0 {
            philosopher(i);
            sys_exit(0);
        } else {
            pids[i] = pid;
        }
    }
    for pid in pids {
        sys_wait_pid(pid);
    }
    0
}

fn philosopher(idx: usize) {
    let pid = sys_get_pid();

    for round in 0..20 {
        // 固定时间思考
        let think_time = 150;
        println!(
            "Philosopher {} is thinking (pid {}, round {}, {} ticks)",
            idx, pid, round, think_time
        );
        delay(think_time);

        // println!("Philosopher {} is hungry (pid {}, round {})", idx, pid, round);
        let left = idx;
        let right = (idx + 1) % N;

        // 服务生法避免死锁
        WAITER.wait();

        CHOPSTICK[left].wait();
        // 拿筷子之间可加一点延迟模拟动作
        delay(5);
        CHOPSTICK[right].wait();

        // 固定时间吃饭
        let eat_time = 80;
        println!(
            "Philosopher {} is eating (pid {}, round {}, {} ticks)",
            idx, pid, round, eat_time
        );
        delay(eat_time);

        CHOPSTICK[left].signal();
        CHOPSTICK[right].signal();
        WAITER.signal();

        // println!("Philosopher {} finished eating (pid {}, round {})", idx, pid, round);
    }
}

#[inline(never)]
fn delay(times: u32) {
    for _ in 0..(times * 100) {
        core::hint::spin_loop();
    }
}

entry!(main);