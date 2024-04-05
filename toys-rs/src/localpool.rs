/*! 
# Pool-rs
A simple single-thread object pool.

# TODO
- [ ] compatible with tokio-local
- [x] implement for types with lifetime
- [ ] automatically shrink
- [ ] variable `INIT_SIZE`
*/

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
/// let mut p /*: Pool<'mutcounter, i32> */ = 
///     Pool::with_init(|x|{*x=counter; counter+=1;});
/// assert_eq!(*p.get(), 4);
/// // Now the p.get() is dropped, so it is put into pool.
/// // Another get() will get the same object.
/// assert_eq!(*p.get(), 4);
/// ```
pub struct Pool<'a, T>{ // lifetime for new function
    pool: Vec<*mut T>,
    newfn: Box<dyn FnMut()->*mut T+'a>,
}

impl<'a, T> Pool<'a, T>{
    const INIT_SIZE:usize=4;
    /// Gets an object from pool
    pub fn get(&mut self)->PoolBox<'a, T>{
        match self.pool.pop(){
        Some(x)=>PoolBox{value:x, pool:self},
        None=>PoolBox{value: (*self.newfn)(), pool: self}
        }
    }

    /// Puts object back into pool(unnecessary)
    pub fn put(&mut self, _: PoolBox<T>){} // automatically call drop

    /// Constructs a new object Pool which provides empty `T` objeccts. 
    /// # Unsafe
    /// Object are uninitialized.
    pub unsafe fn new()->Self{
        Self::with_new(||{
            unsafe{mem::new()}
        })
    }

    fn with_new<NewFn: FnMut()->*mut T+'a>(mut newfn: NewFn)->Self{
        let mut pool=Vec::with_capacity(Self::INIT_SIZE);
        for _ in 0..(Self::INIT_SIZE) {
            pool.push(newfn());
        }
        Self { pool, newfn: Box::new(newfn) }
    }

    /// Constructs a new object Pool which provides `T` objeccts
    /// initialized with `init(&mut T)`. 
    /// # Unsafe
    /// Object may be uninitialized.
    pub fn with_init<Init:FnMut(&mut T)+'a> (mut init: Init)->Self{
        Self::with_new(move||{unsafe{
            let p=mem::new();
            init(&mut *p);
            p
        }})
    }

    /// Constructs a new object Pool which provides `T` objeccts
    /// generated with `generate()->T`. 
    pub fn with_generator<Generator:FnMut()->T+'a>(mut generate: Generator)->Self{
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
}


impl<'a, T> Drop for Pool<'a, T>{
    fn drop(&mut self) {
        for x in &self.pool{
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
pub struct PoolBox<'a, T>{
    value: *mut T,
    pool: *mut Pool<'a, T>,
}

impl<'a, T> std::ops::Deref for PoolBox<'a, T>{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: self.value will never be null
        unsafe{& *self.value}
    }
}
impl<'a, T> std::ops::DerefMut for PoolBox<'a, T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: self.value will never be null
        unsafe{&mut *self.value}
    }
}
impl<'a, T> Drop for PoolBox<'a, T>{
    fn drop(&mut self) {unsafe{
        // SAFETY: push is runned on local thread
        (*self.pool).pool.push(self.value);
    }}
}

// SAFETY: the PoolBox can be safely borrowed between threads
unsafe impl<'a, T: Sync> Sync for PoolBox<'a,T>{}

#[cfg(test)]
mod tests {
    #[test]
    fn test_init(){
        use super::*;
        let mut counter=1;
        let mut p=Pool::with_init(|x|{*x=counter; counter+=1;});
        assert_eq!(*p.get(), 4);
        drop(p);
    }

    #[test]
    fn test_value(){
        use super::*;
        let s=String::from("hello");
        let mut p=Pool::with_value(&s);
        assert_eq!(*p.get(), "hello");
    }

    #[test]
    fn test_gen(){
        use super::*;
        let mut x=0;
        let mut p=Pool::with_generator(||{let y=x; x+=1; y});
        assert_eq!(*p.get(), 3);
    }
}

use crate::mem;