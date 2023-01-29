use std::ffi::c_int;

macro_rules! ffi {
    ($($name:ident);+$(;)?) => {
        $(pub fn $name(data: &mut [c_int]) {
            mod ffi {
                use super::*;
                extern "C" {
                    pub fn $name(ptr: *mut c_int, len: c_int);
                }
            }
            unsafe {
                ffi::$name(data.as_mut_ptr(), data.len() as c_int);
            }
        })+
    };
}

ffi! {
    quicksort;
    heapsort;
    insertion_sort;
    counting_sort;
    mysort;
}
