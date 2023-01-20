use std::ffi::c_int;

pub fn insertion_sort(data: &mut [c_int]) {
    extern "C" {
        fn insertion_sort(data: *mut c_int, len: c_int);
    }
    unsafe {
        insertion_sort(data.as_mut_ptr(), data.len() as i32);
    }
}

pub fn mysort(data: &mut [c_int]) {
    extern "C" {
        fn mysort(data: *mut c_int, len: c_int);
    }
    unsafe {
        mysort(data.as_mut_ptr(), data.len() as i32);
    }
}
