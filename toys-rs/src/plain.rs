//! no_std serde tools that minimizes cpu calculation for plain-layout types.
//! 
//! Basic usage:
//! ```rust
//! use toys_rs::plain::{from_bytes, to_bytes};
//! // automatic implement
//! #[repr(C)]
//! #[derive(Debug, Copy, Clone, PartialEq)]
//! struct MyStruct {
//!     a: u32,
//!     b: i64,
//! }
//! 
//! let my_struct = MyStruct { a: 42, b: 100 };
//! let bytes: &[u8] = to_bytes(&my_struct);
//! println!("Bytes: {:?}", bytes); // [42, 0, 0, 0, ?, ?, ?, ?, 100, 0, 0, 0, 0, 0, 0, 0]
//! let my_struct_back = from_bytes::<MyStruct>(bytes);
//! assert_eq!(&*my_struct_back, &my_struct);
//! // points to same place, 
//! assert_eq!(
//!     &*my_struct_back as *const MyStruct as usize, 
//!     &my_struct  as *const MyStruct as usize);
//! 
//! ```
//! 
//! Regarding binary serde, [bincode](https://tyoverby.com/posts/bincode_release.html)(std)
//! and [postcard](https://github.com/jamesmunns/postcard)(no_std) are
//! 2 production-ready choice. They are fast and almost perfect.
//! 
//! The tool has many limitations:
//! - no dynamic-lengthed type support
//! - no endianness compatibility
//! 
//! The only advantage is speed - it barely does anything, just convert type
//! and performs memcpy.

use core::ops::{Deref, DerefMut};

/// types that can safely load from plain bytes array.
/// - No dereference should be included, or it should wrap
///   with `Option`s that clears before converted to bytes
/// - Initial value can be cloned by plain memcopy
pub unsafe trait Plain: 'static {}

// implement `Plain` for most seemingly plain type
unsafe impl<T:Copy+Clone+Sync+Send+'static> Plain for T {}

/// converts a plain object to its corresponding bytes array
#[inline]
pub fn to_bytes<T:Plain>(x: &T)->&[u8] {
    unsafe{core::slice::from_raw_parts(
        x as *const T as *const u8, core::mem::size_of::<T>()
    )}
}

/// converts a bytes array to plain object copy-on-write pointer.
/// 
/// The pointer remains same as input array if alignment is correct,
/// allowing reuse in read-only senario.
#[inline]
pub fn from_bytes<T:Plain>(x: &[u8])->MyCow<T>{
    MyCow::from_bytes(x)
}

/// converts a bytes array to plain object copy-on-write pointer.
/// 
/// The pointer remains same as input array if alignment is correct,
/// allowing reuse of mutable mem space.
#[inline]
pub fn from_bytes_mut<T:Plain>(x: &mut [u8])->MyCow<T>{
    MyCow::from_bytes_mut(x)
}

pub enum MyCow<'a, T:Plain>{
    Borrow(&'a T),
    BorrowMut(&'a mut T),
    Owned(T),
}

impl<'a,T:Plain> Deref for MyCow<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        match self {
            Self::Borrow(x) => x,
            Self::BorrowMut(x) => x,
            Self::Owned(x) => x,
        }
    }
}

impl<'a, T:Plain> DerefMut for MyCow<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        match self {
            Self::Borrow(p)=>{
                let p=*p;
                *self=Self::Owned(unsafe{core::ptr::read(p)});
                if let Self::Owned(x)=self {x} else {panic!()} // never
            },
            Self::BorrowMut(x)=>*x,
            Self::Owned(x)=>x
        }
    }
}

impl<'a, T:Plain> MyCow<'a, T> {
    /// consumes the cow pointer and returns inner value
    pub fn into_inner(self)->T {
        match self {
            Self::Borrow(p)=>unsafe{core::ptr::read(p)},
            Self::BorrowMut(p)=>unsafe{core::ptr::read(p)},
            Self::Owned(x)=>x
        }
    }

    fn from_bytes(x: &[u8])->Self {
        if core::mem::size_of::<T>()!=x.len() {
            panic!("size not match!");
        }
        if core::mem::align_of::<T>() != 1 && 
        (x.as_ptr() as usize) % core::mem::align_of::<T>() !=0 {
            // alignment mismatch
            MyCow::Owned(unsafe{core::ptr::read_unaligned(x.as_ptr() as *const T)})
        } else {
            MyCow::Borrow(unsafe{&*(x.as_ptr() as *const T)})
        }
    }

    fn from_bytes_mut(x: &mut [u8])->Self {
        if core::mem::size_of::<T>()!=x.len() {
            panic!("size not match!");
        }
        if core::mem::align_of::<T>() != 1 && 
        (x.as_ptr() as usize) % core::mem::align_of::<T>() !=0 {
            // alignment mismatch
            MyCow::Owned(unsafe{core::ptr::read_unaligned(x.as_ptr() as *const T)})
        } else {
            MyCow::BorrowMut(unsafe{&mut *(x.as_mut_ptr() as *mut T)})
        }
    }
}