use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Result}, 
    net::{TcpStream, TcpListener},
    sync::broadcast::Receiver
};
use std::net::SocketAddr;
use log::{info, warn, debug, error, trace};
use crate::responses::*;

pub async fn handle<T: Clone>(listener: TcpListener, mut alive: Receiver<T>){
    loop{tokio::select! {
        conn = listener.accept() => {
            if let Ok((stream, addr)) = conn  {
                debug!("Accept connection from from {addr}");
                tokio::spawn(async move {
                    let result = handle_conn(stream, addr).await;
                    #[cfg(debug_assertions)]
                    trace!("connection closed");
                    result
                });
            } else {
                return;
            }
        },
        _ = alive.recv() =>{
            return;
        }
    }}
}

pub async fn handle_conn(mut stream: TcpStream, addr: SocketAddr) ->Result<()>{
    let mut request=Vec::new();
    loop{
        let mut lines = BufReader::new(&mut stream).lines();
        while let Some(line)=lines.next_line().await?{
            if line.is_empty(){ // request end
                break;
            }
            request.push(line);
        }
        if request.is_empty() { // finished
            break Ok(())
        }
        #[cfg(debug_assertions)]
        trace!("got request {:#?}", request);
        let index;
        stream.write({
            let s=unsafe{request.first().unwrap_unchecked()};
            let s=s.to_lowercase();
            if s.starts_with("get / "){
                info!("Request from {addr}");
                index=generate_index(addr.ip().to_string());
                index.as_bytes()
            } else if s.starts_with("get /"){
                warn!("URI {} not found, request from {addr}", unsafe{s.split_whitespace().nth(1).unwrap_unchecked()});
                NOT_FOUND
            } else {
                error!("Bad Request `{s}` from {addr}");
                BAD_REQ
            }
        }).await?;
        request.clear();
    }
}