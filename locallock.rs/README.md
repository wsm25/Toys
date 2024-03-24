# Local Lock
An async waiting lock for single-thread asynchronized context.

# Basic Usage
```rust
async fn main(){
    let lock = LocalLock::new();
    spawn_local(dummy(lock.clone())).join();
    spawn_local(dummy(lock.clone())).join();
}

async fn dummy(lock: LocalLock){
    lock.lock().await;
    ... // some async tasks
    // unneeded as drop has been implemented
    // lock.unlock(); 
}
```
# Warning
Unlock will be automatically called when lock is dropped. If you don't need lock, don't clone it.
# TODO
- implement locker counter, so no auto unlock when drop if lock unused