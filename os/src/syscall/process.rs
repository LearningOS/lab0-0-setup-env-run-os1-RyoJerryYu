use alloc::sync::Arc;

use crate::{
    loaders::get_app_data_by_name,
    mm::{translated_refmut, translated_str},
    println,
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next,
    },
    timer::get_time_ms,
};

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next(exit_code);
    unreachable!("Unreachable after sys_exit");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    let new_trap_cx = new_task.inner_xclusive_access().get_trap_cx();
    // return value for child process
    // child process directly go to trap_return
    // did not execute here
    new_trap_cx.x[10] = 0;

    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let current_task = current_task().unwrap();
    let mut current_task_inner = current_task.inner_xclusive_access();
    if current_task_inner
        .children
        .iter()
        .find(|p| pid == -1 || pid as usize == p.getpid())
        .is_none()
    {
        return -1;
    }

    let pair = current_task_inner
        .children
        .iter()
        .enumerate()
        .find(|(_, p)| {
            p.inner_xclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        });

    if let Some((index, _)) = pair {
        let child = current_task_inner.children.remove(index);
        assert_eq!(Arc::strong_count(&child), 1);

        let found_pid = child.getpid();
        let exit_code = child.inner_xclusive_access().exit_code;

        *translated_refmut(current_task_inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
}
