use tokio::{
    time::{Duration, sleep},
    runtime::Builder,
    task::{LocalSet, spawn_local},
};
use rand::random;
use super::*;
#[test]
fn test_lock(){
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