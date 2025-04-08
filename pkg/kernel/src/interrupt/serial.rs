use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::serial::get_serial_for_sure;
use crate::drivers::input::push_key;

use super::consts::*;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Serial0 as u8]
        .set_handler_fn(serial_handler);
}

pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
    receive();
    super::ack();
}

/// Receive character from uart 16550
/// Should be called on every interrupt
fn receive() {
    // FIXME: receive character from uart 16550, put it into INPUT_BUFFER
    let mut serial = get_serial_for_sure(); // 获取串口
    let c = serial.receive(); // 接收字符
    drop(serial);
    if let Some(c) = c {
        push_key(c); // 把字符放进 input 缓冲区
    }
}