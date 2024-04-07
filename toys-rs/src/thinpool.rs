//! A simple single-thread object pool, allocated in bundle.

struct ThinBundle<T>{ // SAFETY: must be pinned
    space: [T; usize::BITS as usize], // [T; 64]
    idle: usize, // idling flat, 1 for idle and 0 for occupied
    pool: *mut Vec<*mut ThinBundle<T>>,
}

struct RawPool<'a, T>{ // lifetime for new function
    pool: Vec<*mut ThinBundle<T>>, // never include fully occupied bundle
    newfn: Box<dyn FnMut()->*mut ThinBundle<T>+'a>
}

#[derive(Clone)]
pub struct Pool<'a, T>
    (Rc<UnsafeCell<RawPool<'a, T>>>);

pub struct PoolBox<T>{
    space: *mut T,
    bundle: *mut ThinBundle<T>,
}

impl<'a, T> Pool<'a, T>{
    fn inner(&self)->&mut RawPool<'a, T>{
        // SAFETY: within one thread, only one mut can be got
        unsafe{&mut *self.0.get()}
    }

    /// Gets an object from pool
    pub fn get(&mut self)->PoolBox<T>{
        let x=self.inner();
        match x.pool.last_mut(){
        None=>unsafe{ // new and return
            let bundle=(*x.newfn)();
            let space=(*bundle).space.as_mut_ptr();
            (*bundle).idle=(!0)^1; // 0b1..10
            (*bundle).pool=&mut x.pool;
            let output=PoolBox{space, bundle};
            x.pool.push(bundle);
            output
        },
        Some(bundle)=>unsafe{
            let bundle=*bundle;
            // SAFETY: idle is never 0
            let offset=(*bundle).idle.trailing_zeros() as usize;
            let space=(*bundle).space.as_mut_ptr().add(offset);
            (*bundle).idle^=1<<offset;
            if (*bundle).idle==0{
                x.pool.pop(); // SAFETY: idle cannot be 0
            }
            PoolBox{space, bundle}
        }}
    }

    /// Puts object back into pool(unnecessary)
    pub fn put(&mut self, _: PoolBox<T>){} // automatically call drop

    fn with_new<New>(newfn: New)->Self
        where New:FnMut()->*mut ThinBundle<T>+'a{
        let pool=Vec::new();
        Pool(Rc::new(UnsafeCell::new(RawPool{ pool, newfn: Box::new(newfn)}))) 
    }

    /// Constructs a new object Pool which provides empty `T` objeccts. 
    /// # Unsafe
    /// Object are uninitialized.
    pub unsafe fn new()->Self{
        Self::with_new(|| mem::new())
    }

    /// Constructs a new object Pool which provides `T` objeccts
    /// initialized with `init(&mut T)`. 
    /// # Unsafe
    /// Object may be uninitialized.
    pub fn with_init<Init> (mut init: Init)->Self
        where Init:FnMut(&mut T)+'a{
        Self::with_new(move||{unsafe{
            let bundle: *mut ThinBundle<T>=mem::new();
            let space=(*bundle).space.as_mut_ptr();
            for i in 0..(usize::BITS as usize){
                init(&mut *space.add(i));
            }
            bundle
        }})
    }

    /// Constructs a new object Pool which provides `T` objeccts
    /// generated with `generate()->T`. 
    pub fn with_generator<Generator>(mut generate: Generator)->Self
        where Generator:FnMut()->T+'a{
        Self::with_new(move||{unsafe{
            let bundle: *mut ThinBundle<T>=mem::new();
            let space=(*bundle).space.as_mut_ptr();
            for i in 0..(usize::BITS as usize){
                space.add(i).write(generate());
            }
            bundle
        }})
    }
    
    /// Constructs a new object Pool which provides `T` objeccts
    /// cloned from `value:T where T:Clone`. 
    /// 
    /// SAFETY: value should outlive pool itself
    pub fn with_value(value: T)->Self where T:Clone+'a{
        Self::with_generator(move || value.clone())
    }

    pub fn idle(&self)->usize{
        self.inner().pool.iter().map(|x|unsafe{(**x).idle.count_ones() as usize}).sum()
    }
}

/// drop will only be called when Rc counter returns 0
impl<'a, T> Drop for RawPool<'a, T>{
    fn drop(&mut self) {
        for x in &self.pool{
            unsafe{mem::delete(*x)}
        }
    }
}


impl<T> std::ops::Deref for PoolBox<T>{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: self.value will never be null
        unsafe{& *self.space}
    }
}
impl<T> std::ops::DerefMut for PoolBox<T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: self.value will never be null
        unsafe{&mut *self.space}
    }
}
impl<T> Drop for PoolBox<T>{
    fn drop(&mut self) {unsafe{
        // SAFETY: push is runned on local thread
        let bundle=self.bundle;
        let offset=self.space.offset_from((*bundle).space.as_ptr());
        if (*bundle).idle==0 {
            (*(*bundle).pool).push(bundle);
        }
        (*self.bundle).idle^=1<<offset;
    }}
}

// SAFETY: the PoolBox can be safely borrowed between threads
unsafe impl<T: Sync> Sync for PoolBox<T>{}


#[allow(deprecated)]
#[cfg(test)]
mod tests {
    
    // #[test]
    fn _test_init(){
        use super::*;
        let mut counter=1;
        let mut p = Pool::with_init(
            |x|{*x=counter; counter+=1;}
        );
        let g1=p.get();
        let g2=p.get();
        assert_eq!(*g1, 1);
        assert_eq!(*g2, 2);
        drop(p);
    }

    // #[test]
    fn _test_value(){
        use super::*;
        let s=String::from("hello");
        let mut p:Pool<_>=Pool::with_value(&s);
        assert_eq!(*p.get(), "hello");
    }

    // #[test]
    fn _test_gen(){
        use super::*;
        let mut x=1;
        let mut p:Pool<_>=Pool::with_generator(||{let y=x; x+=1; y});
        assert_eq!(*p.get(), 1);
    }

    // #[test]
    fn _test_clone(){
        use super::*;
        let p:Pool<i32>=Pool::with_init(|_|{});
        let mut p1=p.clone();
        drop(p1.get()); // make sure p1 not stripped
    }

    
    // #[test]
    fn _test_tokio(){
        use tokio::{
            runtime::Builder,
            task::{LocalSet, spawn_local, yield_now},
        };
        use super::*;
        async fn sleepygreeting<'a>(mut pool: Pool<'a, i32>){
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
    
}

use std::{cell::UnsafeCell, rc::Rc};

use crate::mem;