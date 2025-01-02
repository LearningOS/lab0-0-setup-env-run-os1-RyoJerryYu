use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::println;

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

lazy_static! {
    static ref APP_NAMES: Vec<&'static str> = {
        let num_app = get_num_app();
        extern "C" {
            fn _app_names();
        }
        let mut start = _app_names as usize as *const u8;
        let mut v = Vec::new();
        unsafe {
            for _ in 0..num_app {
                let mut end = start;
                while end.read_volatile() != 0 {
                    end = end.add(1);
                }
                let slice = core::slice::from_raw_parts(start, end as usize - start as usize);
                let str = core::str::from_utf8(slice).unwrap();
                v.push(str);
                start = end.add(1);
            }
        }
        v
    };
}

pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
    APP_NAMES
        .iter()
        .position(|&x| x == name)
        .map(|id| get_app_data(id))
}

pub fn list_apps() {
    println!("/*** Apps ***/");
    for app in APP_NAMES.iter() {
        println!("{}", app);
    }
    println!("/*** End ***/");
}
