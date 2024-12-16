use core::arch::asm;

use crate::config::*;
use crate::mm::KERNEL_SPACE;
use crate::trap::context::TrapContext;
use crate::trap::trap_handler;

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

// allocate a space in the data segment
static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];
static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];

impl KernelStack {
    // it will be used when initialize
    // otherwize, we do not get the sp from this method
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

// get the base address allocate for the app memory space
fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

// get the number of apps
// as in link_app.S , the val at _num_app is the number of apps
pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

// get the start of app_starts
// as in link_app.S , [app_start_0, app_start_1, ... , app_start_n, app_end_n]
// is at [_num_app + 1, _num_app + 2, ... , _num_app + n + 1]
fn get_app_start_arr_ptr() -> *const usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).add(1) }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    let num_app = get_num_app();
    let app_start_arr_ptr = get_app_start_arr_ptr();
    let app_start = unsafe { core::slice::from_raw_parts(app_start_arr_ptr, num_app + 1) };
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    }
}

pub fn load_apps() {
    // load the app_start and num_app
    // just as same as the init of APP_MANAGER in ch2
    let num_app = get_num_app();
    let app_start_arr_ptr = get_app_start_arr_ptr();
    let app_start = unsafe {
        // app_0_start, app_1_start, ... , app_n_start, app_n_end
        core::slice::from_raw_parts(app_start_arr_ptr, num_app + 1)
    };

    for i in 0..num_app {
        // for each app, load it to the memory space
        // just as same as the AppManager::load_app in ch2
        let base_i = get_base_i(i);
        // clear the memory space
        (base_i..base_i + APP_SIZE_LIMIT)
            .for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });
        // load the app to the memory space
        let app_src = unsafe {
            core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i])
        };
        let dst = unsafe { core::slice::from_raw_parts_mut(base_i as *mut u8, app_src.len()) };
        dst.copy_from_slice(app_src);
        unsafe {
            asm!("fence.i");
        }
    }
}

// init the KERNEL_STACK for the app
pub fn init_app_cx(app_id: usize) -> usize {
    KERNEL_STACK[app_id].push_context(TrapContext::app_init_context(
        get_base_i(app_id),
        USER_STACK[app_id].get_sp(),
        KERNEL_SPACE.exclusive_access().token(),
        0, // TODO: set the kernel_sp
        trap_handler as usize,
    )) as *const _ as usize
}

/// init batch subsystem
pub fn init() {
    load_apps();
}
