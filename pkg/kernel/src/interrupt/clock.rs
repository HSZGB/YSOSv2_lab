use core::sync::atomic::{AtomicU64, Ordering};

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::memory::gdt;
use crate::proc::*;
use super::consts::*;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
        .set_handler_fn(clock_handler)
        .set_stack_index(gdt::CLOCK_INTERRUPT_IST_INDEX);
}

pub extern "C" fn clock(mut context: ProcessContext) {
    // debug!("Timer interrupt triggered");
    
    // do something
    switch(&mut context);
    super::ack();
}

as_handler!(clock);

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn read_counter() -> u64 {
    // FIXME: load counter value
    COUNTER.load(Ordering::Relaxed)
}

#[inline]
pub fn inc_counter() -> u64 {
    // FIXME: read counter value and increase it
    COUNTER.fetch_add(1, Ordering::Relaxed) + 1
}