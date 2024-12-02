use core::arch::global_asm;
use context::TrapContext;
use log::trace;
use riscv::register::{scause::{self, Exception, Trap}, stval, stvec, utvec::TrapMode};

use crate::{println, syscall};

pub mod context;


global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }

    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            trace!("[kernel] UserEnvCall, syscall id = {}", cx.x[17]);
            cx.sepc += 4;      
            cx.x[10] = syscall::syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", stval, cx.sepc);
            panic!("[kernel] Cannot continue!");
            // run_next_app();
        } 
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            panic!("[kernel] Cannot continue!");
            //run_next_app();
        }
        _=> {
            panic!("unhandled trap: {:?}, stval = {:#x}!\n", scause.cause(), stval);
        }
    }
    cx
}