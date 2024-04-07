//! Elegant wrapper of [std::alloc]
use std::alloc::{alloc, dealloc, realloc, Layout};

/// Allocates memory for type `T` without initilization.
/// 
/// # Safety
/// - MUST deallocate when dropping or clearing all its referees
/// - MUST initialize fields before visiting
/// - See [std::alloc::GlobalAlloc::alloc]
#[inline(always)]
pub unsafe fn new<T>()->*mut T{
    alloc(Layout::new::<T>()) as *mut T
}

/// Allocates for type `T` and initialize it.
/// # Safety
/// MUST deallocate when dropping or clearing all its referees.
#[inline(always)]
pub unsafe fn from<T>(x:T)->*mut T{
    let p=new::<T>();
    p.write(x);
    p
}

/// Allocates memory for array `[T;n]` without initilization.
/// 
/// # Safety
/// See [new]
#[inline(always)]
pub unsafe fn new_arr<T>(n: usize)->*mut T{
    alloc(Layout::array::<T>(n).unwrap()) as *mut T
}

/// Deallocates memory for pointer `ptr`.
/// # Safety
/// This function is only supposed to use in `fn drop`, to cowork and 
/// make use of Rust's ownership and lifetime. Otherwise, please make
/// sure you will never use it.
/// 
/// - See [std::alloc::GlobalAlloc::dealloc]
/// - You MUST manually drop its fields if extra allocation exists.
///   This function only deallocates memory for `T` itself
/// - Never use this function on array pointer (see tips below) or stack pointer
/// 
/// # Tips
/// ## Array pointer
/// If you want to delete array pointer, you have two choices:
/// 1. use [delete_arr], with raw pointer and length
/// 2. use [delete_slice], with slice fat pointer
/// 
/// Both of them requires length information.
#[inline(always)]
pub unsafe fn delete<T>(ptr: *mut T){
    dealloc(ptr as *mut u8, Layout::new::<T>())
}

/// Deallocates memory for an array pointer with size `n`.
/// # Safety
/// See [delete]
#[inline(always)]
pub unsafe fn delete_arr<T>(ptr: *mut T, n: usize){
    dealloc(ptr as *mut u8, Layout::array::<T>(n).unwrap())
}

/// Deallocates memory for a slice reference.
/// # Safety
/// See [delete]
#[inline(always)]
pub unsafe fn delete_slice<T>(slice: &mut [T]){
    dealloc(slice.as_mut_ptr() as *mut u8, Layout::for_value(slice))
}

/// Resize the array pointer to given length. 
/// 
/// Note that if return pointer is null, that means the new object
/// could not be allocated in place, thus old pointer should remains.
/// 
/// # Safety
/// You MUST correctly handle null pointer returned.
/// 
/// And see [std::alloc::GlobalAlloc::realloc]
#[inline(always)]
pub unsafe fn resize_arr<T>(ptr: *mut T, oldlen: usize, newlen: usize)->*mut T{
    realloc(ptr as *mut u8, 
        Layout::array::<T>(oldlen).unwrap(),
        Layout::array::<T>(newlen).unwrap().size())
    as *mut T
}

/// Returns a `*mut T` null pointer
#[inline(always)]
pub const fn nullptr<T>()->*mut T{0 as *mut T}
