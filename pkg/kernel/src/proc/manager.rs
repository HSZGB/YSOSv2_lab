use super::*;
use crate::{filesystem::get_rootfs, humanized_size, memory::{
    self,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure, PAGE_SIZE,
}, proc::vm::stack::STACK_INIT_TOP, resource::Resource};
use alloc::{collections::*, format, sync::{Arc, Weak}};
use spin::{Mutex, RwLock};
use storage::FileSystem;

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>, app_list: boot::AppListRef) {

    // FIXME: set init process as Running
    init.write().resume();

    // FIXME: set processor's current pid to init's pid
    processor::set_pid(init.pid());

    PROCESS_MANAGER.call_once(|| ProcessManager::new(init, app_list));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
    app_list: boot::AppListRef,
    wait_queue: Mutex<BTreeMap<ProcessId, BTreeSet<ProcessId>>>,
}

impl ProcessManager {
    pub fn new(init: Arc<Process>, app_list: boot::AppListRef) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let wait_queue = BTreeMap::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
            app_list: app_list,
            wait_queue: Mutex::new(wait_queue),
        }
    }

    pub fn app_list(&self) -> boot::AppListRef {
        self.app_list
    }

    

    #[inline]
    pub fn push_ready(&self, pid: ProcessId) {
        self.ready_queue.lock().push_back(pid);
    }

    #[inline]
    fn add_proc(&self, pid: ProcessId, proc: Arc<Process>) {
        self.processes.write().insert(pid, proc);
    }

    #[inline]
    pub fn get_proc(&self, pid: &ProcessId) -> Option<Arc<Process>> {
        self.processes.read().get(pid).cloned()
    }

    pub fn current(&self) -> Arc<Process> {
        self.get_proc(&processor::get_pid())
            .expect("No current process")
    }

    pub fn save_current(&self, context: &ProcessContext) {
        // FIXME: update current process's tick count
        self.current().write().tick();

        // FIXME: save current process's context
        self.current().write().save(context);
    }

    pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {

        // 如何使这一函数的功能仅限于“切换到下一个进程”

        let mut ready_queue = self.ready_queue.lock();

        // FIXME: fetch the next process from ready queue
        // FIXME: check if the next process is ready,
        //        continue to fetch if not ready
        if let Some(next_pid) = ready_queue.pop_front() {
            if let Some(next_proc) = self.get_proc(&next_pid) {
                if next_proc.read().status() == ProgramStatus::Ready {
                    // FIXME: restore next process's context
                    next_proc.write().restore(context);
                    // FIXME: update processor's current pid
                    processor::set_pid(next_pid);
                    // FIXME: return next process's pid
                    return next_pid;
                }
            }
        }
        get_pid()
    }

    // pub fn spawn_kernel_thread(
    //     &self,
    //     entry: VirtAddr,
    //     name: String,
    //     proc_data: Option<ProcessData>,
    // ) -> ProcessId {
    //     let kproc = self.get_proc(&KERNEL_PID).unwrap();
    //     let page_table = kproc.read().clone_page_table(); // 内存空间和内核是共享的
    //     let proc_vm = Some(ProcessVm::new(page_table));
    //     let proc = Process::new(name, Some(Arc::downgrade(&kproc)), proc_vm, proc_data);

    //     let pid = proc.pid();
    //     // alloc stack for the new process base on pid
    //     let stack_top = proc.alloc_init_stack();

    //     // FIXME: set the stack frame
    //     proc.write().init_stack_frame(entry, stack_top);

    //     // FIXME: add to process map
    //     self.add_proc(pid, proc);

    //     // FIXME: push to ready queue
    //     self.push_ready(pid);

    //     // FIXME: return new process pid
    //     pid
    // }

    pub fn spawn(
        &self,
        elf: &ElfFile,
        name: String,
        parent: Option<Weak<Process>>,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc_vm = Some(ProcessVm::new(page_table));
        let proc = Process::new(name, parent, proc_vm, proc_data);
    
        let mut inner = proc.write();
        // FIXME: load elf to process pagetable
        inner.load_elf(elf);

        debug!("load elf success.");

        // FIXME: alloc new stack for process
        inner.init_stack_frame(VirtAddr::new_truncate(elf.header.pt2.entry_point()), VirtAddr::new_truncate(STACK_INIT_TOP));

        // FIXME: mark process as ready
        inner.pause();

        drop(inner);
    
        trace!("New {:#?}", &proc);
    
        let pid = proc.pid();
        // FIXME: something like kernel thread
        self.add_proc(pid, proc);
        self.push_ready(pid);
    
        pid
    }

    pub fn fork(&self) {
        // FIXME: get current process
        let current = self.current();

        // FIXME: fork to get child
        let child_proc = current.fork();
        
        // FIXME: add child to process list
        self.add_proc(child_proc.pid(), child_proc.clone());
        self.push_ready(child_proc.pid());

        // FOR DBG: maybe print the process ready queue?
    }

    /// Block the process with the given pid
    pub fn block(&self, pid: ProcessId) {
        if let Some(proc) = self.get_proc(&pid) {
            // FIXME: set the process as blocked
            proc.write().block();
        }
    }

    pub fn wait_pid(&self, pid: ProcessId) {
        let mut wait_queue = self.wait_queue.lock();
        // FIXME: push the current process to the wait queue
        //        `processor::get_pid()` is waiting for `pid`
        wait_queue.entry(pid).or_default().insert(get_pid());
    }

    /// Wake up the process with the given pid
    ///
    /// If `ret` is `Some`, set the return value of the process
    pub fn wake_up(&self, pid: ProcessId, ret: Option<isize>) {
        if let Some(proc) = self.get_proc(&pid) {
            let mut inner = proc.write();
            if let Some(ret) = ret {
                // FIXME: set the return value of the process
                //        like `context.set_rax(ret as usize)`
                inner.set_return_value(ret)
            }
            // FIXME: set the process as ready
            // FIXME: push to ready queue
            inner.pause();
            self.push_ready(pid);
        }
    }

    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault

        // 越权访问
        if err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) {
            warn!("Page fault: writing to address {:#x}", addr);
            return false;
        }

        return self.current().write().handle_page_fault(addr);
    }

    pub fn kill(&self, pid: ProcessId, ret: isize) {
        let proc = self.get_proc(&pid);

        if proc.is_none() {
            warn!("Process #{} not found.", pid);
            return;
        }

        let proc = proc.unwrap();

        if proc.read().status() == ProgramStatus::Dead {
            warn!("Process #{} is already dead.", pid);
            return;
        }

        trace!("Kill {:#?}", &proc);

        proc.kill(ret);

        if let Some(pids) = self.wait_queue.lock().remove(&pid) {
            for pid in pids {
                self.wake_up(pid, Some(ret));
            }
        }
    }

    pub fn print_process_list(&self) {
        let mut output = String::from("  PID | PPID | Process Name |  Ticks  |   Memory  | Status\n");

        self.processes
            .read()
            .values()
            .filter(|p| p.read().status() != ProgramStatus::Dead)
            .for_each(|p| output += format!("{}\n", p).as_str());

        // TODO: print memory usage of kernel heap
        


        let alloc = get_frame_alloc_for_sure();
        let frames_used = alloc.frames_used();
        let frames_recycled = alloc.frames_recycled();
        let frames_total = alloc.frames_total();

        let used = (frames_used - frames_recycled) * PAGE_SIZE as usize;
        let total = frames_total * PAGE_SIZE as usize;

        output += &format_usage("Memory", used, total);
        drop(alloc);

        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }

    pub fn get_exit_code(&self, pid: ProcessId) -> Option<isize> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            self.get_proc(&pid).and_then(|proc| proc.read().exit_code())
        })
        // 如果该值为 None，则说明进程还没有退出
        // 如果该值为 Some，则说明进程已经退出，可以获取到进程的返回值。
    }

    pub fn open(&self, path: &str) -> Option<u8> {
        let handle = match get_rootfs().open_file(path) {
            Ok(handle) => handle,
            Err(e) => {
                warn!("Failed to open file '{}': {:?}", path, e);
                return None;
            }
        };
        Some(self.current().write().open(Resource::File(handle)))
    }

    pub fn close(&self, fd: u8) -> bool {
        self.current().write().close(fd)
    }

}

// A helper function to format memory usage
fn format_usage(name: &str, used: usize, total: usize) -> String {
    let (used_float, used_unit) = humanized_size(used as u64);
    let (total_float, total_unit) = humanized_size(total as u64);

    format!(
        "{:<6} : {:>6.*} {:>3} / {:>6.*} {:>3} ({:>5.2}%)\n",
        name,
        2,
        used_float,
        used_unit,
        2,
        total_float,
        total_unit,
        used as f32 / total as f32 * 100.0
    )
}