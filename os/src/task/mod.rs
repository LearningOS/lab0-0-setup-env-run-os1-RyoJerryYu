use lazy_static::lazy_static;
use task::TaskControlBlock;

use crate::{config::MAX_APP_NUM, loaders::get_num_app, sync::UPSafeCell};

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
        let mut tasks = [TaskControlBlock{
            task_status: task::TaskStatus::UnInit,
            task_cx: context::TaskContext::zero_init(),
        }; MAX_APP_NUM];
        
        for (i, task) in tasks.iter_mut().enumerate() {
            //TODO: goto_restore for each task
            _ = i;
            task.task_status = task::TaskStatus::Ready;
        }

        TaskManager{
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner{
                    tasks,
                    current_task: 0,
                })
            }
        }
    };
}
