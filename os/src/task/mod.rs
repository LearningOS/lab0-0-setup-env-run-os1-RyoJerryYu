use alloc::sync::Arc;
use context::TaskContext;
use lazy_static::lazy_static;
use processor::{schedule, take_current_task};
use task::TaskControlBlock;

use crate::fs::{open_file, OpenFlags};

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

    let mut current_task_inner = current_task.inner_exclusive_access();
    let current_task_cx_ptr = &mut current_task_inner.task_cx as *mut context::TaskContext;
    current_task_inner.task_status = task::TaskStatus::Ready;
    drop(current_task_inner);

    add_task(current_task);
    schedule(current_task_cx_ptr);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    let task = take_current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.task_status = task::TaskStatus::Zombie;
    inner.exit_code = exit_code;

    // move children to INITPROC
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            let mut child_inner = child.inner_exclusive_access();
            child_inner.parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }

    inner.children.clear();
    inner.memory_set.recycle_data_pages();
    drop(inner);
    drop(task);

    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut TaskContext);
}

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = {
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        Arc::new(TaskControlBlock::new(v.as_slice()))
    };
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}
