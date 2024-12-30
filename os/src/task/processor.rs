use alloc::sync::Arc;
use lazy_static::lazy_static;

use crate::{sync::UPSafeCell, trap::TrapContext};

use super::{
    context::TaskContext,
    manager::fetch_task,
    switch,
    task::{TaskControlBlock, TaskStatus},
};

pub struct Processor {
    // reference to the current running task
    current: Option<Arc<TaskControlBlock>>,
    // the task context of the idle task
    // the idle task is the task that the processor runs when there is no other task to run
    // mostly it is the loop in run_tasks
    idle_task_cx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Processor {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    /// return the current task
    /// leave the current to be None
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(|t| t.clone())
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

pub fn run_tasks() -> ! {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(next_task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            let mut next_task_inner = next_task.inner_xclusive_access();
            let next_task_cx_ptr = &mut next_task_inner.task_cx as *mut TaskContext;
            next_task_inner.task_status = TaskStatus::Running;
            drop(next_task_inner);

            processor.current = Some(next_task);
            drop(processor);

            // if the task is the first time to run
            // it will return to the trap_return
            // defined in TaskControlblock::new
            unsafe {
                switch::__switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        }
    }
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

pub fn current_user_token() -> usize {
    current_task()
        .map(|task| task.inner_xclusive_access().get_user_token())
        .unwrap()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_xclusive_access()
        .get_trap_cx()
}

/// switch from switched_task to the idle task
/// that it will return to run_tasks after the switch
/// and it will continue to run the next loop and fetch the next task
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);

    unsafe {
        switch::__switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
