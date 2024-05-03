//! A simple single-thread object pool. As size for single
//! object is fixed, chunks and bins in malloc are too heavy.
//! 
//! maintains a virtual memory mapping, enableing
//! - use of memory fragments
//! - prefer lower address
//! - can automatically dealloc high address

pub struct Heap<T>{
    raw: *mut T,
    top: *mut T,
    cap: usize,
    idle: BinaryHeap<NonNull<T>>,
}

impl<T> Heap<T>{
    pub fn new(cap: usize)->Self{
        let raw=unsafe{crate::mem::new_arr(cap)};
        Self { raw , top: raw, cap , idle:BinaryHeap::new()}
    }
    pub fn get(&mut self)->Option<NonNull<T>>{
        match self.idle.is_empty(){
            false=>self.idle.pop(),
            true=>{
                if (self.top as usize- self.raw as usize)<self.cap{
                    let ptr=Some(unsafe{NonNull::new_unchecked(self.top)});
                    self.top=unsafe{self.top.add(1)};
                    ptr
                } else {None}
            }
        }
    }
    // UNSAFE: ptr MUST belong to this heap
    pub fn put(&mut self, ptr: NonNull<T>){
        if ptr.as_ptr() == unsafe{self.top.sub(1)}{
            self.top=unsafe{self.top.sub(1)};
            while let Some(ptr)=self.idle.peek(){
                if ptr.as_ptr() != unsafe{self.top.sub(1)} {break;}
                self.top=unsafe{self.top.sub(1)};
                self.idle.pop();
            }
        } else {
            self.idle.push(ptr);
        }
    }
}

impl<T> Drop for Heap<T>{
    fn drop(&mut self) {
        unsafe{crate::mem::delete_arr(self.raw, self.cap)};
    }
}

#[cfg(test)]
mod tests {
    
    #[test]
    fn _test_init(){
        use super::*;
        let mut p = Heap::<i32>::new(10);
        let g1=p.get().unwrap();
        let g2=p.get().unwrap();
        println!("{}",*unsafe{g1.as_ref()});
        println!("{}",*unsafe{g2.as_ref()});
        drop(p);
    }
    
    /*
    // #[test]
    fn _test_tokio(){
        use tokio::{
            runtime::Builder,
            task::{LocalSet, spawn_local, yield_now},
        };
        use super::*;
        async fn sleepygreeting(mut pool: Pool<i32>){
            for _ in 0..5{
                let x=pool.get();
                if true==rand::random(){
                    yield_now().await;
                }
                println!("Get {} from pool!", *x);
            }
        }
        async fn tokio_main(){
            let mut ipool=0;
            let pool = Pool::with_generator(move||{ipool+=1; ipool});
            let mut tasks = Vec::new();
            for _ in 0..5{
                tasks.push(spawn_local(
                    sleepygreeting(pool.clone())
                ));
            }
            for t in tasks{
                let _ = t.await;
            }
        }
        Builder::new_current_thread().enable_time().build().unwrap().block_on(
            LocalSet::new().run_until(tokio_main())
        );
    }    
    */
}

use std::{collections::BinaryHeap, ptr::NonNull};