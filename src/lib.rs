use std::ffi::c_int;

mod ffi {
    use std::ffi::c_int;
    extern "C" {
        pub(super) fn heapsort(data: *mut c_int, len: c_int);
        pub(super) fn insertion_sort(data: *mut c_int, len: c_int);
        pub(super) fn mysort(data: *mut c_int, len: c_int);
    }
}

pub fn insertion_sort(data: &mut [c_int]) {
    unsafe {
        ffi::insertion_sort(data.as_mut_ptr(), data.len() as i32);
    }
}

pub fn heapsort(data: &mut [c_int]) {
    unsafe {
        ffi::heapsort(data.as_mut_ptr(), data.len() as i32);
    }
}

pub fn mysort(data: &mut [c_int]) {
    unsafe {
        ffi::mysort(data.as_mut_ptr(), data.len() as i32);
    }
}
