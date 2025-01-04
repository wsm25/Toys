//! Broken mutex lock for rust async in single-thread context

use std::{
    cell::UnsafeCell, 
    collections::VecDeque, 
    future::Future, 
    pin::Pin, 
    rc::Rc, 
    task::{Context, Poll, Waker}
};

struct RawLocalLock{
    queue: VecDeque<Waker>,
    locked: bool,
}
/// # Basic Usage
/// ```rust, ignore 
/// use toys_rs::locallock::LocalLock;
/// #[tokio::main(flavor = "current_thread")]
/// async fn main(){
///     let lock = LocalLock::new();
///     spawn_local(dummy(lock.clone()));
/// }
/// 
/// async fn dummy(lock: LocalLock){
///     lock.lock().await;
///     ... // some async tasks
///     // unneeded as drop has been implemented
///     // lock.unlock(); 
/// }
/// ```
/// # Warning
/// Unlock will be automatically called when lock is dropped. If you don't need lock, don't clone it.
/// # TODO
/// - implement locker counter, so no auto unlock when drop if lock unused
#[derive(Clone)]
pub struct LocalLock{
    lockcell: Rc<UnsafeCell<RawLocalLock>>
}
#[repr(transparent)]
pub struct LocalLocker{lock: LocalLock}

impl LocalLock{
    pub fn new()->Self{
        Self{
            lockcell: Rc::new(UnsafeCell::new(RawLocalLock{
                queue: VecDeque::new(),
                locked: false,
            }))
        }
    }

    fn get(&mut self)->&mut RawLocalLock{
        unsafe{&mut *self.lockcell.get()}
    }

    pub fn lock(&mut self)->&mut LocalLocker{
        unsafe{std::mem::transmute(self)}
    }

    pub fn try_lock(&mut self)->bool{
        let l = self.get();
        if l.locked{
            false
        } else {
            l.locked=true;
            true
        }
    }

    fn try_unlock(&mut self)->bool{
        let l = self.get();
        if l.locked{
            l.locked=false;
            if let Some(waker) = l.queue.pop_front(){
                let (len, cap) = (l.queue.len(), l.queue.capacity()/3);
                if len>0x100 && len<cap{ // if >4k and < 1/3, auto shrink to 2/3
                    l.queue.shrink_to(cap*2);
                }
                waker.wake();
            }
            true
        }else{
            false
        }
    }
    pub fn unlock(&mut self){
        if !self.try_unlock(){
            panic!("unlock unlocked lock!");
        }
    }
}

impl Drop for LocalLock{
    fn drop(&mut self) {
        self.try_unlock();
    }
}

impl Default for LocalLock {
    fn default() -> Self {
        Self::new()
    }
}

impl Future for LocalLocker{
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let l = self.lock.get();
        if l.locked {
            l.queue.push_back(cx.waker().clone());
            Poll::Pending
        } else {
            l.locked=true;
            Poll::Ready(())
        }
    }
}


#[cfg(test)]
mod test{
    use tokio::{
        time::{Duration, sleep},
        runtime::Builder,
        task::{LocalSet, spawn_local},
    };
    use rand::random;
    use super::*;
    // #[test]
    fn _test_lock(){
        async fn sleepygreeting(mut lock: LocalLock, x: isize){
            lock.lock().await;
            sleep(Duration::from_nanos(random::<u64>()%0x1000)).await;
            println!("Greetings from {x}!");
        }
        async fn tokio_main(){
            let lock = LocalLock::new();
            let mut tasks = Vec::new();
            for i in 0..10{
                tasks.push( spawn_local(
                    sleepygreeting(lock.clone(), i)
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