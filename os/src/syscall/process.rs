use crate::println;

pub fn sys_exit(exit_code: usize) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    // run_next_app();
    panic!("Unreachable after sys_exit");
}