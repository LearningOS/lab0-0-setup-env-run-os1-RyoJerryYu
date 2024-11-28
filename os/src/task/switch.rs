use core::arch::global_asm;

use super::context::TaskContext;

global_asm!(include_str!("switch.S"));

extern "C" {
    pub fn __switch(
        current_task_ctx_ptr: *mut TaskContext,
        next_task_ctx_ptr: *const TaskContext,
    );
}