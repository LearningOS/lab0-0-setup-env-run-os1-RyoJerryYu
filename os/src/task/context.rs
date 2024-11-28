#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12], // s0-s11
}

impl TaskContext {
    pub fn zero_init() -> Self {
        TaskContext { ra: 0, sp: 0, s: [0; 12] }
    }
}