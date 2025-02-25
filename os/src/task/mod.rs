use alloc::sync::Arc;
use context::TaskContext;
use lazy_static::lazy_static;
use manager::remove_from_pid2task;
use processor::{schedule, take_current_task};
use task::TaskControlBlock;

use crate::{
    fs::{open_file, OpenFlags},
    println,
    sbi::shutdown,
};

mod action;
mod context;
mod manager;
mod pid;
mod processor;
mod signal;
mod switch;
mod task;

pub use action::{SignalAction, SignalActions};
pub use manager::add_task;
pub use manager::pid2task;
pub use processor::{current_task, current_trap_cx, current_user_token, run_tasks};
pub use signal::{SignalFlags, MAX_SIG};

pub fn suspend_current_and_run_next() {
    let current_task = take_current_task().unwrap();

    let mut current_task_inner = current_task.inner_exclusive_access();
    let current_task_cx_ptr = &mut current_task_inner.task_cx as *mut context::TaskContext;
    current_task_inner.task_status = task::TaskStatus::Ready;
    drop(current_task_inner);

    add_task(current_task);
    schedule(current_task_cx_ptr);
}

pub const IDLE_PID: usize = 0;

pub fn exit_current_and_run_next(exit_code: i32) {
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        println!("The idle task exit with exit_code {}", exit_code);
        if exit_code != 0 {
            shutdown(true);
        } else {
            shutdown(false);
        }
    }

    remove_from_pid2task(task.getpid());
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

pub fn check_signals_error_of_current() -> Option<(i32, &'static str)> {
    let task = current_task().unwrap();
    let task_inner = task.inner_exclusive_access();
    task_inner.signals.check_error()
}

pub fn current_add_signal(signal: SignalFlags) {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.signals |= signal;
}

fn call_kernel_signal_handler(signal: SignalFlags) {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    match signal {
        SignalFlags::SIGSTOP => {
            inner.frozen = true;
            inner.signals.toggle(SignalFlags::SIGSTOP);
        }
        SignalFlags::SIGCONT => {
            if inner.signals.contains(SignalFlags::SIGCONT) {
                inner.signals.toggle(SignalFlags::SIGCONT);
                inner.frozen = false;
            }
        }
        _ => {
            inner.killed = true;
        }
    }
}

fn call_user_signal_handler(sig: usize, signal: SignalFlags) {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();

    let handler = inner.signal_actions.table[sig].handler;
    if handler == 0 {
        // default action
        println!("[K] task/call_user_signal_handler: default action: ignore it or kill process");
        return;
    }

    // user handler

    inner.handling_sig = sig as isize;
    inner.signals.toggle(signal);

    // backup trap context
    let trap_ctx = inner.get_trap_cx();
    inner.trap_ctx_backup = Some(*trap_ctx);

    trap_ctx.sepc = handler; // set pc
    trap_ctx.x[10] = sig; // put a0
}

fn check_pending_signals() {
    for sig in 0..(MAX_SIG + 1) {
        let task = current_task().unwrap();
        let task_inner = task.inner_exclusive_access();
        let signal = SignalFlags::from_bits(1 << sig).unwrap();
        if task_inner.signals.contains(signal) && (!task_inner.signal_mask.contains(signal)) {
            let mut masked = true;
            let handling_sig = task_inner.handling_sig;
            if handling_sig == -1 {
                // not handling
                masked = false;
            } else {
                let handling_sig = handling_sig as usize;
                if !task_inner.signal_actions.table[handling_sig]
                    .mask
                    .contains(signal)
                {
                    // handling but not masked
                    masked = false;
                }
            }

            if !masked {
                drop(task_inner);
                drop(task);
                if signal == SignalFlags::SIGKILL
                    || signal == SignalFlags::SIGSTOP
                    || signal == SignalFlags::SIGCONT
                    || signal == SignalFlags::SIGDEF
                {
                    // signal is a kernel signal
                    call_kernel_signal_handler(signal);
                } else {
                    // signal is a user signal
                    call_user_signal_handler(sig, signal);
                }
            }
        }
    }
}

pub fn handle_signals() {
    loop {
        check_pending_signals();
        let (frozen, killed) = {
            let task = current_task().unwrap();
            let inner = task.inner_exclusive_access();
            (inner.frozen, inner.killed)
        };
        if !frozen || killed {
            break;
        }
        suspend_current_and_run_next();
    }
}
