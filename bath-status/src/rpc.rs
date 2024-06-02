use tokio::sync::mpsc::{self, Receiver, Sender};
use std::{future::Future, ops::DerefMut, pin::Pin, task::{Context, Poll}};
enum Status{
    Born,
    Running,
    Runned,
}
struct Task<T>{
    status: Status,
    cmd: T,
    tx: Sender<PTask<T>>,
}
impl<T> Unpin for Task<T> {}

// assume 
struct PTask<T>(*mut Task<T>, );
// SAFETY: not safe at all XD
// assume status is pinned
unsafe impl<T: Send> Send for PTask<T>{}
impl<T> PTask<T>{
    fn new(p: &mut Task<T>)->Self{
        Self(p)
    }
    fn inner(&self)->&mut T{
        unsafe{&mut (*self.0).cmd}
    }
    fn sync(&self){
        unsafe{(*self.0).status=Status::Runned}
    }
}

impl<T> Future for Task<T>{
    type Output = Result<(), ()>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use mpsc::error::TrySendError::*;
        use Status::*;
        use Poll::*;
        match self.status{
        Born=>{
            let p=PTask::new(&mut self);
            match self.tx.try_send(p){
            Ok(_)=>{
                self.status=Running;
                Pending
            },
            Err(e)=>{match e{
                Full(_)=>Pending,
                Closed(_)=>Ready(Err(())) // full
            }}
            }
        },
        Running=>Pending,
        Runned=>Ready(Ok(()))
        }
    }
}

struct Rpc<T>{
    tx: Sender<PTask<T>>,
}

trait RpcHandler<T>{
    fn execute(&mut self, cmd: &mut T);
}

async fn rpc_daemon<T>(mut rx: Receiver<PTask<T>>, mut h: impl RpcHandler<T>){
    while let Some(task)=rx.recv().await{
        h.execute(task.inner());
        task.sync();
    }
}

impl<T: Send> Rpc<T>{
    /// @param rpc_ddaemon: an async function receving data. MUST modify status after edit
    pub fn new<F, Fut>(rpc_daemon: F)->Self
        where F:FnOnce(Receiver<PTask<T>>)->Fut + Send,
            Fut: Future<Output = ()>{
        let (tx, rx)=mpsc::channel(256);
        std::thread::spawn(move|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .build().unwrap()
                .block_on(
                tokio::task::LocalSet::new()
                    .run_until(rpc_daemon(rx))
            )
        }).join().unwrap();

        Rpc { tx }
    }

    pub fn run(&mut self, cmd: T)->Task<T>{
        Task { status: Status::Born, cmd, tx: self.tx.clone()}
    }
}