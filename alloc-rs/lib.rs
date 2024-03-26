use std::alloc::{alloc, dealloc, Layout};

#[inline(always)]
pub unsafe fn new<T>()->*mut T{
    alloc(Layout::new::<T>()) as *mut T
}

/// never use on array pointer
#[inline(always)]
pub unsafe fn delete<T>(ptr: *mut T){
    dealloc(ptr as *mut u8, Layout::for_value(&*ptr))
}

#[inline(always)]
pub unsafe fn new_arr<T>(n: usize)->*mut T{
    alloc(Layout::array::<T>(n).unwrap()) as *mut T
}

/// with runtime list searching cost
#[inline(always)]
pub unsafe fn delete_arr<T>(ptr: *mut T){
    dealloc(ptr as *mut u8, Layout::for_value(&*ptr))
}

#[inline(always)]
pub unsafe fn delete_arr_sized<T>(ptr: *mut T, n: usize){
    dealloc(ptr as *mut u8, Layout::array::<T>(n).unwrap())
}
