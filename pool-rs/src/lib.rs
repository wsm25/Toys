/*! 
# Pool-rs
A simple single-thread object pool.

# TODO
- compatible with tokio-local
*/

pub struct Pool<T>{
    pool: Vec<*mut T>,
    newfn: *const (dyn Fn()->*mut T+'static)
}

impl<T> Pool<T>{
    const INIT_SIZE:usize=4;
    pub fn get(&mut self)->PoolBox<T>{
        match self.pool.pop(){
        Some(x)=>PoolBox{value:x, pool: self},
        None=>PoolBox{value: unsafe{(*self.newfn)()}, pool: self}
        }
    }

    pub fn put(&mut self, _x: PoolBox<T>){} // automatically call drop
    pub fn new()->Self{
        Self::with_new_closure(unsafe{mem::from(||{
            mem::new()
        })})
    }

    fn with_new_closure(newfn: *const (dyn Fn()->*mut T))->Self{
        let mut pool=Vec::with_capacity(Self::INIT_SIZE);
        for _ in 0..(Self::INIT_SIZE) {
            pool.push(unsafe{(*newfn)()});
        }
        Self { pool, newfn }
    }

    pub fn with_init<Init:Fn(&mut T)+'static> (init: Init)->Self{
        Self::with_new_closure(unsafe{mem::from(move||{
            let p=mem::new();
            init(&mut *p);
            p
        })})
    }

    pub fn with_value<Init:Fn()->T+'static>(init: Init)->Self{
        Self::with_new_closure(unsafe{mem::from(move||{
            let p:*mut T=mem::new();
            p.write(init());
            p
        })})
    }

    
}

impl<T> Drop for Pool<T>{
    fn drop(&mut self) {
        for x in &self.pool{
            // SAFETY: pointer won't dealloc until Pool is dropped
            unsafe{mem::delete(*x)}
        }
    }
}

pub struct PoolBox<T>{
    value: *mut T,
    pool: *mut Pool<T>,
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
        (*self.pool).pool.push(self.value);
    }}
}

unsafe impl<T: Send> Send for PoolBox<T>{}
unsafe impl<T: Sync> Sync for PoolBox<T>{}

#[cfg(test)]
mod tests {
    #[test]
    fn test_init(){
        use super::*;
        let counter=unsafe{mem::from(1)}; // BUG: mem leak
        let mut p: Pool<i32>=Pool::with_init(move|x|{unsafe{
            *x=*counter; *counter+=1;
        }});
        assert_eq!(*p.get(), 4);
    }
}

mod mem;