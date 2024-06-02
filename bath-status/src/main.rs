mod database;
mod webapi;
mod rpc;

fn main(){
    std::thread::spawn(||{
        tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build().unwrap()
            .block_on(
            tokio::task::LocalSet::new()
                .run_until(webapi::daemon())
        )
    }).join().unwrap();
}