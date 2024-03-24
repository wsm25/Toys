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
/// ```rust
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

    pub fn lock(&mut self)->& mut LocalLocker{
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
mod test;