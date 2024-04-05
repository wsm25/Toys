# Local Lock
An async waiting lock for single-thread asynchronized context.

# Basic Usage
```toml
# Cargo.toml
[dependencies]
local_lock = { git = "https://github.com/wsm25/Toys", path = "locallock.rs"}
```
```rust
use local_lock::LocalLock;
async fn main(){
    let lock = LocalLock::new();
    spawn_local(dummy(lock.clone()));
    spawn_local(dummy(lock.clone()));
    wait_tasks();
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