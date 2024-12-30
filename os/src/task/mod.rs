use alloc::{sync::Arc, vec::Vec};
use context::TaskContext;
use lazy_static::lazy_static;
use processor::{schedule, take_current_task};
use task::TaskControlBlock;

use crate::{
    loaders::{get_app_data, get_app_data_by_name, get_num_app},
    println,
    sbi::shutdown,
    sync::UPSafeCell,
    trap::context::TrapContext,
};

mod context;
mod manager;
mod pid;
mod processor;
mod switch;
mod task;

pub use manager::add_task;
pub use processor::{current_task, current_trap_cx, current_user_token, run_tasks};

pub fn suspend_current_and_run_next() {
    let current_task = take_current_task().unwrap();

    let mut current_task_inner = current_task.inner_xclusive_access();
    let current_task_cx_ptr = &mut current_task_inner.task_cx as *mut context::TaskContext;
    current_task_inner.task_status = task::TaskStatus::Ready;
    drop(current_task_inner);

    add_task(current_task);
    schedule(current_task_cx_ptr);
}

pub fn exit_current_and_run_next() {
    let mut inner = self.inner.exclusive_access();
    let current_task = inner.current_task; // tips: use a variable to avoid borrow checker error
    inner.tasks[current_task].task_status = task::TaskStatus::Exited;
    run_next_task();
}

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("initproc").unwrap(),
    ));
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}
