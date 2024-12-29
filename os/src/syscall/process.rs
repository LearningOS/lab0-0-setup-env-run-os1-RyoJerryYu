use crate::{
    println,
    task::{add_task, current_task, exit_current_and_run_next, suspend_current_and_run_next},
    timer::get_time_ms,
};

pub fn sys_exit(exit_code: usize) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    unreachable!("Unreachable after sys_exit");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
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
