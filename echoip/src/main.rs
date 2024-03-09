use tokio::{
    net::TcpListener,
    sync::broadcast,
};
use log::info;

// modules;
mod responses;
mod handler;

#[tokio::main]
async fn main() {
    // logger
    #[cfg(debug_assertions)]
    env_logger::builder().format_target(false).parse_env(
        env_logger::Env::default().default_filter_or("debug")).init();
    #[cfg(not(debug_assertions))]
    env_logger::builder().format_target(false).parse_env(
        env_logger::Env::default().default_filter_or("info")).init();
    // bind port argv[1]
    let port=std::env::args().nth(1).unwrap_or("7878".to_string());

    // linux binds both v4 and v6 on [::]
    #[cfg(target_os = "linux")]
    {
        let (tx, rx)=broadcast::channel(1);
        let handler = tokio::spawn(handler::handle(
            TcpListener::bind(":::".to_string()+&port).await
                .unwrap_or_else(|_|{panic!("error bind to [::]:{}",port)}),
            rx
        ));
        info!("Dual-stack handler start on :{}", port);
        let _ = tokio::signal::ctrl_c().await;
        info!("Receive SIGINT, exiting...");
        let _= (tx.send(()), handler.await);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let (tx, rx4)=broadcast::channel(1);
        let handler4 = tokio::spawn(handler::handle(
            TcpListener::bind("0.0.0.0:".to_string()+&port).await
                .unwrap_or_else(|_|{panic!("error bind to 0.0.0.0:{}",port)}),
            rx4
        ));
        info!("IPv4 handler start on 0.0.0.0:{}", port);
        let handler6 = tokio::spawn(handler::handle(
            TcpListener::bind(":::".to_string()+&port).await
                .unwrap_or_else(|_|{panic!("error bind to [::]:{}",port)}),
            tx.subscribe()
        ));
        info!("IPv6 handler start on [::]:{}", port);
        let _ = tokio::signal::ctrl_c().await;
        info!("Receive SIGINT, exiting...");
        let _= (tx.send(()), handler4.await, handler6.await);
    }
    
}
