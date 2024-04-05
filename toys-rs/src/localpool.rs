/*! A simple single-thread object pool.
# TODO
- [x] compatible with tokio-local
- [x] implement for types with lifetime
- [ ] automatic shrink

Abandoned:
- DST support: deprecated with Box<_>
- variable `INIT_SIZE`: rust still has inferring bugs for generic const
*/

const INIT_SIZE:usize=8; // init size

struct RawPool<'a, T>{ // lifetime for new function
    pool: Vec<*mut T>,
    newfn: Box<dyn FnMut()->*mut T+'a>
}

#[derive(Clone)]
pub struct Pool<'a, T>
    (Rc<UnsafeCell<RawPool<'a, T>>>);

/// An object pool that generates and stores [PoolBox].
/// 
/// It will automatically generate new `T` object, with support to
/// types and environments with bounded lifetime.
/// 
/// The memory only releases when the pool itself is dropped.
/// 
/// # Safety
/// ## Maybe Uninit
/// We allow minimal initialization of the object, so objects may be 
/// uninitialized. Please initialize before visiting.
/// 
/// ## Thread Safety
/// It is not thread-safe.
/// 
/// # Example
/// ```rust
/// use toys_rs::localpool::Pool;
/// let mut counter = 1;
/// let mut p : Pool = 
///     Pool::with_init(|x|{*x=counter; counter+=1;});
/// assert_eq!(*p.get(), 8);
/// // Now the p.get() is dropped, so it is put into pool.
/// // Another get() will get the same object.
/// assert_eq!(*p.get(), 8);
/// ```

impl<'a, T> Pool<'a, T>{
    fn inner(&self)->&mut RawPool<'a, T>{
        // SAFETY: within one thread, only one mut can be got
        unsafe{&mut *self.0.get()}
    }

    /// Gets an object from pool
    pub fn get(&mut self)->PoolBox<T>{
        let p=self.inner();
        match p.pool.pop(){
        Some(x)=>PoolBox{value:x, pool:&mut p.pool},
        None=>PoolBox{value: (*p.newfn)(), pool: &mut p.pool}
        }
    }

    /// Puts object back into pool(unnecessary)
    pub fn put(&mut self, _: PoolBox<T>){} // automatically call drop

    fn with_new<New>(mut newfn: New)->Self
        where New:FnMut()->*mut T+'a{
        let mut pool=Vec::with_capacity(INIT_SIZE);
        for _ in 0..INIT_SIZE {
            pool.push(newfn());
        }
        Pool(Rc::new(UnsafeCell::new(RawPool{ pool, newfn: Box::new(newfn)}))) 
    }

    /// Constructs a new object Pool which provides empty `T` objeccts. 
    /// # Unsafe
    /// Object are uninitialized.
    pub unsafe fn new()->Self{
        Self::with_new(||{
            unsafe{mem::new()}
        })
    }

    /// Constructs a new object Pool which provides `T` objeccts
    /// initialized with `init(&mut T)`. 
    /// # Unsafe
    /// Object may be uninitialized.
    pub fn with_init<Init> (mut init: Init)->Self
        where Init:FnMut(&mut T)+'a{
        Self::with_new(move||{unsafe{
            let p=mem::new();
            init(&mut *p);
            p
        }})
    }

    /// Constructs a new object Pool which provides `T` objeccts
    /// generated with `generate()->T`. 
    pub fn with_generator<Generator>(mut generate: Generator)->Self
        where Generator:FnMut()->T+'a{
        Self::with_new(move||{unsafe{
            mem::from(generate())
        }})
    }
    
    /// Constructs a new object Pool which provides `T` objeccts
    /// cloned from `value:T where T:Clone`. 
    /// 
    /// SAFETY: value should outlive pool itself
    pub fn with_value(value: T)->Self where T:Clone+'a{
        Self::with_new(move||{unsafe{
            mem::from(value.clone())
        }})
    }

    pub fn idle(&self)->usize{self.inner().pool.len()}
}

/// drop will only be called when Rc counter returns 0
impl<'a, T> Drop for RawPool<'a, T>{
    fn drop(&mut self) {
        for x in &self.pool{
            // println!("dropping poolbox {:#?}", *x);
            // SAFETY: pointer won't dealloc until Pool is dropped
            // SAFETY: each pointer will only appear once as `PoolBox`s
            // are generated and dropped only once.
            unsafe{mem::delete(*x)}
        }
    }
}

/// Smart pointer that belongs to a [Pool].
/// 
/// It will be automatically put to the owner pool when dropped,
/// and will be reused when `pool.get()` is called.
/// 
/// # Safety
/// You MUST initialize before read, which is not guaranteed
pub struct PoolBox<T>{
    value: *mut T,
    pool: *mut Vec<*mut T>, // as return only happens when used,
                            // extra one dereference is accepted
                            // with 2 usize less space as reward
}

impl<T> std::ops::Deref for PoolBox<T>{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: self.value will never be null
        unsafe{& *self.value}
    }
}
impl<T> std::ops::DerefMut for PoolBox<T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: self.value will never be null
        unsafe{&mut *self.value}
    }
}
impl<T> Drop for PoolBox<T>{
    fn drop(&mut self) {unsafe{
        // SAFETY: push is runned on local thread
        (*self.pool).push(self.value);
    }}
}

// SAFETY: the PoolBox can be safely borrowed between threads
unsafe impl<T: Sync> Sync for PoolBox<T>{}

#[cfg(test)]
mod tests {
    #[test]
    fn test_init(){
        use super::*;
        let mut counter=1;
        let mut p = Pool::with_init(
            |x|{*x=counter; counter+=1;}
        );
        assert_eq!(*p.get(), INIT_SIZE);
        drop(p);
    }

    #[test]
    fn test_value(){
        use super::*;
        let s=String::from("hello");
        let mut p:Pool<_>=Pool::with_value(&s);
        assert_eq!(*p.get(), "hello");
    }

    #[test]
    fn test_gen(){
        use super::*;
        let mut x=1;
        let mut p:Pool<_>=Pool::with_generator(||{let y=x; x+=1; y});
        assert_eq!(*p.get(), INIT_SIZE);
    }

    #[test]
    fn test_clone(){
        use super::*;
        let p:Pool<i32>=unsafe{Pool::new()};
        let mut p1=p.clone();
        drop(p1.get()); // make sure p1 not stripped
    }

    /*
    #[test]
    fn test_tokio(){
        use tokio::{
            time::{Duration, sleep},
            runtime::Builder,
            task::{LocalSet, spawn_local},
        };
        use rand::random;
        use super::*;
        async fn sleepygreeting(mut lock: LocalLock, x: isize){
            lock.lock().await;
            sleep(Duration::from_nanos(random::<u64>()%0x1000)).await;
            println!("Greetings from {x}!");
        }
        async fn tokio_main(){
            let pool = unsafe{Pool::new()};
            let mut tasks = Vec::new();
            for i in 0..10{
                tasks.push(spawn_local(
                    sleepygreeting(pool.clone(), i)
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

use std::{cell::UnsafeCell, rc::Rc};

use crate::mem;