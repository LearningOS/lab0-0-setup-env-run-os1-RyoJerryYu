#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12], // s0-s11
}
extern "C" {
    fn __restore(cx_addr: usize);
}

impl TaskContext {
    pub fn zero_init() -> Self {
        TaskContext {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    pub fn goto_restore(kstack_ptr: usize) -> Self {
        TaskContext {
            ra: __restore as usize,
            sp: kstack_ptr + core::mem::size_of::<TaskContext>(),
            s: [0; 12],
        }
    }
}
