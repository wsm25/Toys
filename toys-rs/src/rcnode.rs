//! Provide loop-safe nodelist implemented with `Rc`
//! 
//! # Basic concept
//! The "next" pointer in memory indicates connection, thus 
//! sharing lifespan with "value". However, the "handlers" as 
//! local variable in essence "owns" both the pointer and 
//! pointer->next.
//! 
//! Consider the classic example:
//! ```rust, ignore
//! use std::rc::Rc;
//! use std::cell::Cell;
//! struct Node{next: Cell<Option<Rc<Node>>>}
//! let x=Rc::new(Node{next:Cell::new(None)});
//! let y=Rc::new(Node{next:Cell::new(Some(x.clone()))});
//! x.next.set(Some(y.clone()));
//! ```
//! 
//! This code will cause memory leak as strong counter of both
//! x and y will remain 1 after drop, thus keeping allocation until exit.
//! 
//! What we expect is that, as x, y both stores cloned `Rc`, it should
//! drop the next when dropping itself whatever its strong counter.
//! 
//! Therefore, we introduce strong and weak nodes. The former owns
//! both itself and next, working as handler; while the latter owns
//! only itself, working as "next" indicator.
//! 
//! Nevertheless, we do not expose weak node structure, as it does not
//! work in general and is highly customized for node type.
//! 
//! # TODO
//! - implement peer list with Vec<Weak<T>>
//! - complete document
//! 
//! # Unsafe
//! This module is **extramely unsafe** to use on types that
//! dynamically allocates memory - it is totally pointless.
//! who wants a graph 
use std::{cell::UnsafeCell, ops::{Deref, DerefMut}, rc::Rc};

struct RawNode<T>{
    value: T,
    next: Option<Weak<T>>,
}

struct Weak<T>(Rc<UnsafeCell<RawNode<T>>>);
impl<T> Clone for Weak<T>{fn clone(&self) -> Self 
    {Self(self.0.clone())}}

impl<T> Weak<T>{
    fn inner(&self)->&mut RawNode<T>{
        // SAFETY(1): assume mut value works fine
        unsafe{&mut *self.0.get()}
    }
    fn new(value: T)->Self{
        Self(Rc::new(UnsafeCell::new(RawNode{
            value, next:None
        })))
    }
    fn with_next(value: T, next: Self)->Self{
        Self(Rc::new(UnsafeCell::new(RawNode{
            value, next:Some(next)
        })))
    }
    fn stronger(&self)->&Node<T>{
        // SAFETY(10): same layout
        unsafe{&*(self as *const Self as *const Node<T>)}
    }
    fn ref_count(&self)->usize{
        Rc::strong_count(&self.0)
    }
}

/// A smart pointer that stores a value and its next node.
/// 
/// # Example
/// ```rust
/// use toys_rs::rcnode::Node;
/// let mut x=Node::new(1); // Node<i32>
/// let y=x.clone();
/// *x+=1; // auto deref
/// assert_eq!(2, *y);
/// let mut y=Node::new(4);
/// x.point(&mut y); // x will point to y
/// let xnext=x.next().unwrap(); // will return a clone of x.next
/// assert_eq!(4, *xnext);
/// ```
pub struct Node<T>(Weak<T>);

impl<T> Node<T>{
    fn inner(&self)->&mut RawNode<T>{
        self.0.inner()
    }
    pub fn new(value: T)->Self{
        Self(Weak::new(value))
    }
    pub fn with_next(value: T, next: &Self)->Self{
        Self(Weak::with_next(value, next.0.clone()))
    }
    pub fn point(&mut self, next: &Self){
        self.inner().next=Some(next.0.clone());
    }
    pub fn point_null(&mut self){
        self.inner().next=None;
    }
    pub fn ref_count(&self)->usize{
        self.0.ref_count()
    }
    pub fn next(&self)->Option<Self>{
        match &self.inner().next{
        None=>None,
        Some(next)=>Some(next.stronger().clone())
        }
    }
}


impl<T> Clone for Node<T>{
    fn clone(&self) -> Self {
        let new=self.0.clone();
        if let Some(n)=&new.inner().next{
            // SAFETY(10): will decrease by drop_in_place in drop
            std::mem::forget(n.0.clone()); // counter+=1
        }
        Self(new)
    }
}

impl<T> Drop for Node<T>{
    fn drop(&mut self) {
        if let Some(n)=&mut self.inner().next{
            // SAFETY(10): next won't live shorter than self
            // as its counter is always larger than self
            unsafe{std::ptr::drop_in_place(n)}; // counter-=1;
        }
    }
}

impl<T> Deref for Node<T>{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner().value
    }
}

impl<T> DerefMut for Node<T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner().value
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_rc_loop(){
        use std::rc::Rc;
        use std::cell::Cell;
        struct Node{next: Cell<Option<Rc<Node>>>}
        let x=Rc::new(Node{next:Cell::new(None)});
        let y=Rc::new(Node{next:Cell::new(Some(x.clone()))});
        x.next.set(Some(y.clone()));
    }
    #[test]
    fn clone() {
        use super::*;
        let mut x=Node::new(1);
        let y=x.clone();
        *x+=1;
        assert_eq!(2, *y);
        assert_eq!(2, x.ref_count());
        drop(y);
        assert_eq!(1, x.ref_count());
    }
    #[test]
    fn cycling(){
        use super::*;
        let mut x=Node::new(1);
        let mut y=Node::with_next(2, &mut x);
        x.point(&mut y);
        assert_eq!(2, x.ref_count());
        assert_eq!(2, y.ref_count());
        drop(y);
        assert_eq!(1, x.ref_count());
    }
    #[test]
    fn next(){
        use super::*;
        let mut x=Node::new(1);
        let mut y=Node::with_next(2, &mut x);
        x.point(&mut y);
        let ynext=y.next().unwrap();
        assert_eq!(3, x.ref_count());
        assert_eq!(3, y.ref_count());
        drop(ynext);
        assert_eq!(2, x.ref_count());
        assert_eq!(2, y.ref_count());
        drop(y);
        assert_eq!(1, x.ref_count());
    }
}