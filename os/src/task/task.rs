use core::cell::RefMut;

use alloc::{rc::Weak, sync::Arc, vec::Vec};

use crate::{
    config::TRAP_CONTEXT,
    mm::{MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE},
    sync::UPSafeCell,
    trap::{context::TrapContext, trap_handler},
};

use super::{
    context::TaskContext,
    pid::{pid_alloc, KernelStack, PidHandle},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
    // Exited removed
    // because at now, a process exited the PCB will be freed
}

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    // mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,    // the memory space mapping of the task
    pub trap_cx_ppn: PhysPageNum, // reserved for trap handler
    pub base_size: usize,         // size for loading elf
    pub parent: Option<Weak<TaskControlBlock>>, // parent task weak reference
    pub children: Vec<Arc<TaskControlBlock>>, // children task owned reference
    pub exit_code: usize,         // exit code for waitpid
}

impl TaskControlBlock {
    pub fn inner_xclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);

        // trap context is const in virtual memory
        // we are in kernel space, so we should use translate
        // to get the physical page number of TRAP_CONTEXT
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap() // must success because TRAP_CONTEXT already mapped in from_elf
            .ppn();
        let task_status = TaskStatus::Ready;

        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();

        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    task_status,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    memory_set,
                    trap_cx_ppn,
                    base_size: user_sp, // user_sp is the top of user stack
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                })
            },
        };

        // it's in kernel space, so it's safe to get mutable reference
        let trap_cx = task_control_block.inner.exclusive_access().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );

        task_control_block
    }
    pub fn exec(&self, elf_data: &[u8]) {
        todo!()
    }
    pub fn fork(self: &Arc<TaskControlBlock>) -> Self {
        todo!()
    }

    pub fn getpid(&self) -> usize {
        self.pid.0
    }
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }

    pub fn get_status(&self) -> TaskStatus {
        self.task_status
    }

    pub fn is_zombie(&self) -> bool {
        self.task_status == TaskStatus::Zombie
    }
}
