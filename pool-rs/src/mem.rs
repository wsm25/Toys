use std::alloc::{alloc, dealloc, Layout};

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
