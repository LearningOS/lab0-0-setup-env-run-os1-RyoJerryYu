use core::cell::RefMut;

use alloc::{
    sync::{Arc, Weak},
    vec,
    vec::Vec,
};

use crate::{
    config::TRAP_CONTEXT,
    fs::{File, Stdin, Stdout},
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
    pub exit_code: i32,           // exit code for waitpid
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>, // file descriptor table
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    pub fn new(elf_data: &[u8]) -> Self {
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
                    fd_table: vec![
                        Some(Arc::new(Stdin)),  // 0: stdin
                        Some(Arc::new(Stdout)), // 1: stdout
                        Some(Arc::new(Stdout)), // 2: stderr
                    ],
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
        // init a new memory set for the new elf
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        // trap context in new memory set
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        let mut inner = self.inner_exclusive_access();
        inner.memory_set = memory_set;
        inner.trap_cx_ppn = trap_cx_ppn;

        // set the new trap context
        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );
    }

    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let mut parent_inner = self.inner_exclusive_access();

        let child_memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
        // content of child trap context also copied from parent in from_existed_user
        let child_trap_cx_ppn = child_memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn(); // init the page for trap context

        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        // copy fd table
        let mut new_fd_table = Vec::new();
        for fd in parent_inner.fd_table.iter() {
            new_fd_table.push(fd.clone());
        }
        // new tcb on the heap
        let task_control_block = Arc::new(Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    task_status: TaskStatus::Ready,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    memory_set: child_memory_set,
                    trap_cx_ppn: child_trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    parent: Some(Arc::downgrade(self)), // create a weak reference to parent
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: new_fd_table,
                })
            },
        });

        // add child
        parent_inner.children.push(task_control_block.clone());

        // modify kernel_sp in trap_cx
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;

        task_control_block
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

    #[allow(unused)]
    pub fn get_status(&self) -> TaskStatus {
        self.task_status
    }

    pub fn is_zombie(&self) -> bool {
        self.task_status == TaskStatus::Zombie
    }

    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
}
