use context::TaskContext;
use lazy_static::lazy_static;
use task::TaskControlBlock;

use crate::{
    config::MAX_APP_NUM,
    loaders::{get_num_app, init_app_cx},
    sync::UPSafeCell,
};

mod context;
mod switch;
mod task;

pub struct TaskManager {
    num_app: usize,                      // app number will not be changed
    inner: UPSafeCell<TaskManagerInner>, // inner data will be changed
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM], // containing task context and status for each task
    current_task: usize,                    // index of the current running task
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_status: task::TaskStatus::UnInit,
            task_cx: context::TaskContext::zero_init(),
        }; MAX_APP_NUM];

        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_cx = TaskContext::goto_restore(init_app_cx(i));
            task.task_status = task::TaskStatus::Ready;
        }

        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        inner.tasks[0].task_status = task::TaskStatus::Running;
        let next_task_cx_ptr = &mut inner.tasks[0].task_cx as *mut context::TaskContext;
        drop(inner);

        let mut _unused = TaskContext::zero_init();

        unsafe {
            switch::__switch(&mut _unused as *mut context::TaskContext, next_task_cx_ptr);
        }
        unreachable!("Unreachable after switch, unless someone found the _unused TaskContext");
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current_task = inner.current_task; // tips: use a variable to avoid borrow checker error
        (current_task + 1..=current_task + self.num_app)
            .map(|i| i % self.num_app)
            .find(|i| inner.tasks[*i].task_status == task::TaskStatus::Ready)
    }

    fn run_next_task(&self) {
        let next = self.find_next_task();
        if next.is_none() {
            panic!("no task to run");
        }
        let next = next.unwrap();

        let mut inner = self.inner.exclusive_access();
        let current_task = inner.current_task; // tips: use a variable to avoid borrow checker error
        inner.tasks[next].task_status = task::TaskStatus::Running;
        inner.current_task = next;
        let current_task_cx_ptr =
            &mut inner.tasks[current_task].task_cx as *mut context::TaskContext;
        let next_task_cx_ptr = &inner.tasks[next].task_cx as *const context::TaskContext;
        drop(inner);

        unsafe {
            switch::__switch(current_task_cx_ptr, next_task_cx_ptr);
        }
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current_task = inner.current_task; // tips: use a variable to avoid borrow checker error
        inner.tasks[current_task].task_status = task::TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current_task = inner.current_task; // tips: use a variable to avoid borrow checker error
        inner.tasks[current_task].task_status = task::TaskStatus::Exited;
    }
}

pub fn run_first_task() -> !{
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}
