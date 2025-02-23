use alloc::{string::String, sync::Arc, vec::Vec};

use crate::{
    fs::{open_file, OpenFlags},
    mm::{translated_ref, translated_refmut, translated_str},
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
    let new_trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // return value for child process
    // child process directly go to trap_return
    // did not execute here
    new_trap_cx.x[10] = 0;

    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);

    // translate args from user space
    let mut args_vec: Vec<String> = Vec::new();
    loop {
        let arg_str_ptr = *translated_ref(token, args);
        if arg_str_ptr == 0 {
            break;
        }
        args_vec.push(translated_str(token, arg_str_ptr as *const u8));
        unsafe {
            args = args.add(1);
        }
    }

    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        task.exec(all_data.as_slice(), args_vec);
        0
    } else {
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let current_task = current_task().unwrap();
    let mut current_task_inner = current_task.inner_exclusive_access();
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
            p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        });

    if let Some((index, _)) = pair {
        let child = current_task_inner.children.remove(index);
        assert_eq!(Arc::strong_count(&child), 1);

        let found_pid = child.getpid();
        let exit_code = child.inner_exclusive_access().exit_code;

        *translated_refmut(current_task_inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
}
