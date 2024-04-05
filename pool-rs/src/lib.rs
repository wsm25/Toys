/*! 
# Pool-rs
A simple single-thread object pool.

# TODO
- compatible with tokio-local
- implement for types with lifetime
*/

pub struct Pool<'a, T: 'a>{
    pool: Vec<*mut T>,
    newfn: Box<dyn FnMut()->*mut T+'a>,
}

impl<'a, T: 'a> Pool<'a, T>{
    const INIT_SIZE:usize=4;
    pub fn get(&mut self)->PoolBox<'a, T>{
        match self.pool.pop(){
        Some(x)=>PoolBox{value:x, pool:self},
        None=>PoolBox{value: (*self.newfn)(), pool: self}
        }
    }

    pub fn put(&mut self, _x: PoolBox<T>){} // automatically call drop
    pub fn new()->Self{
        Self::with_new_closure(Box::new(||{
            unsafe{mem::new()}
        }))
    }

    fn with_new_closure(mut newfn: Box<dyn FnMut()->*mut T+'a>)->Self{
        let mut pool=Vec::with_capacity(Self::INIT_SIZE);
        for _ in 0..(Self::INIT_SIZE) {
            pool.push((*newfn)());
        }
        Self { pool, newfn }
    }

    pub fn with_init<Init:FnMut(&mut T)+'a> (mut init: Init)->Self{
        Self::with_new_closure(Box::new(move||{unsafe{
            let p=mem::new();
            init(&mut *p);
            p
        }}))
    }

    pub fn with_generator<Generator:FnMut()->T+'a>(mut init: Generator)->Self{
        Self::with_new_closure(Box::new(move||{unsafe{
            mem::from(init())
        }}))
    }
    
    // input value must be 'static. If it is with lifetime, we suppose
    // a clone.
    pub fn with_value(value: T)->Self where T:Clone{
        Self::with_new_closure(Box::new(move||{unsafe{
            mem::from(value.clone())
        }}))
    }
}


impl<'a, T> Drop for Pool<'a, T>{
    fn drop(&mut self) {
        for x in &self.pool{
            println!("droppint pointer {:#?}", *x);
            // SAFETY: pointer won't dealloc until Pool is dropped
            unsafe{mem::delete(*x)}
        }
    }
}

pub struct PoolBox<'a, T:'a>{
    value: *mut T,
    pool: *mut Pool<'a, T>,
}

impl<'a, T:'a> std::ops::Deref for PoolBox<'a, T>{
    type Target = T;
    fn deref(&self) -> &'a Self::Target {
        // SAFETY: self.value will never be null
        unsafe{& *self.value}
    }
}
impl<'a, T:'a> std::ops::DerefMut for PoolBox<'a, T>{
    fn deref_mut(&mut self) -> &'a mut Self::Target {
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
unsafe impl<'a, T: Sync+'a> Sync for PoolBox<'a,T>{}


#[cfg(test)]
mod tests {
    #[test]
    fn test_init(){
        use super::*;
        let mut counter=Box::new(1);
        let mut p=Pool::with_init(move|x|{
            *x=*counter; *counter+=1;
        });
        assert_eq!(*p.get(), 4);
        drop(p);
    }

    #[test]
    fn test_value(){
        use super::*;
        let s=String::from("hello");
        let sref=&s;
        let mut p=Pool::with_value(sref);
        assert_eq!(*p.get(), "hello");
    }
}

mod mem;