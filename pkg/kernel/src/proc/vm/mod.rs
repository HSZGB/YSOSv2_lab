use alloc::format;
use x86_64::{
    structures::paging::{page::*, *},
    VirtAddr,
};
use xmas_elf::ElfFile;

use crate::{humanized_size, memory::*};

pub mod stack;

use self::stack::*;

use super::{PageTableContext, ProcessId};

type MapperRef<'a> = &'a mut OffsetPageTable<'static>;
type FrameAllocatorRef<'a> = &'a mut BootInfoFrameAllocator;

pub struct ProcessVm {
    // page table is shared by parent and child
    pub(super) page_table: PageTableContext,

    // stack is pre-process allocated
    pub(super) stack: Stack,
}

impl ProcessVm {
    pub fn new(page_table: PageTableContext) -> Self {
        Self {
            page_table,
            stack: Stack::empty(),
        }
    }

    pub fn init_kernel_vm(mut self) -> Self {
        // TODO: record kernel code usage
        self.stack = Stack::kstack();
        self
    }

    pub fn init_proc_stack(&mut self, pid: ProcessId) -> VirtAddr {

        // 在本次实验中，笔者带领大家做一个临时的、取巧的实现：根据进程的 PID 来为进程分配对应的栈空间。
        // 也即，对于 PID 为 3 的进程，它的栈空间比 PID 为 2 的进程的栈空间具有 4GiB 的偏移。
        // FIXME: calculate the stack for pid
        // debug!("{}", pid.0);
        // 计算栈底和栈顶地址
        let stack_top = STACK_INIT_TOP - (pid.0 as u64 - 1) * STACK_MAX_SIZE;
        // let stack_bottom = stack_top - STACK_MAX_SIZE;
        let stack_bottom = STACK_INIT_BOT - (pid.0 as u64 - 1) * STACK_MAX_SIZE;

        // 获取页表映射器和帧分配器
        let mapper = &mut self.page_table.mapper();
        let frame_allocator = &mut *get_frame_alloc_for_sure();

        // 映射栈空间
        // 初始是4KB
        elf::map_range(
            stack_bottom,
            STACK_DEF_PAGE, // 页数
            mapper,
            frame_allocator,
        )
        .expect("Failed to map stack range");
        
        // init for stack
        self.stack = stack::Stack::new(
            Page::containing_address(VirtAddr::new(stack_top)),
            STACK_DEF_PAGE,
        );

        // 返回栈顶地址
        VirtAddr::new(stack_top)
    }

    pub fn handle_page_fault(&mut self, addr: VirtAddr) -> bool {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        self.stack.handle_page_fault(addr, mapper, alloc)
    }

    pub(super) fn memory_usage(&self) -> u64 {
        self.stack.memory_usage()
    }
    
    pub fn load_elf(&mut self, elf: &ElfFile) {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();
    
        self.stack.init(mapper, alloc);
    
        // FIXME: load elf to process pagetable
        elf::load_elf(
            elf,
            *PHYSICAL_OFFSET.get().unwrap(), 
            mapper,
            alloc,
            true,
        );

        self.stack.init(mapper, alloc);
    }
}

impl core::fmt::Debug for ProcessVm {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = humanized_size(self.memory_usage());

        f.debug_struct("ProcessVm")
            .field("stack", &self.stack)
            .field("memory_usage", &format!("{} {}", size, unit))
            .field("page_table", &self.page_table)
            .finish()
    }
}
