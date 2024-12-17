use crate::{
    config::TRAP_CONTEXT,
    mm::{MapPermission, MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE},
    println,
    trap::{context::TrapContext, trap_handler},
};

use super::context::TaskContext;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    UnInit,  // 未初始化
    Ready,   // 准备运行
    Running, // 运行中
    Exited,  // 已退出
}

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,    // the memory space mapping of the task
    pub trap_cx_ppn: PhysPageNum, // reserved for trap handler
    pub base_size: usize,         // size for loading elf
}

impl TaskControlBlock {
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

        // map kernel stack
        let (kernel_stack_bottom, kernel_stack_top) = crate::config::kernel_stack_position(app_id);
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );

        let task_control_block = Self {
            task_status,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_cx_ppn,
            base_size: user_sp, // user_sp is the top of user stack
        };

        // it's in kernel space, so it's safe to get mutable reference
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );

        task_control_block
    }

    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
}
