use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Result}, 
    net::{TcpStream, TcpListener},
    sync::broadcast::Receiver,
    select,
};
use std::net::{SocketAddr, IpAddr};
use log::{info, warn, debug, error, trace};
use crate::responses::*;

pub async fn handle<T: Clone>(listener: TcpListener, mut alive: Receiver<T>){
    loop{select! {
        conn = listener.accept() => {
            if let Ok((stream, addr)) = conn  {
                debug!("Accept connection from from {addr}");
                tokio::spawn(async move {
                    let result = handle_conn(stream, addr).await;
                    debug!("connection closed");
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

/// TODO handle keep-alive
pub async fn handle_conn(mut stream: TcpStream, addr: SocketAddr) ->Result<()>{
    let mut request=Vec::new();
    loop{
        let mut lines = BufReader::new(&mut stream).lines();
        while let Some(line)=lines.next_line().await?{
            #[cfg(debug_assertions)]
            trace!("> {line}");
            if line.is_empty(){ // request end
                break;
            }
            request.push(line);
        }
        if request.is_empty() { // finished
            break Ok(())
        }
        
        let index;
        let response = {
            let s=unsafe{request.first().unwrap_unchecked()};
            let s=s.to_lowercase();
            if s.starts_with("get / "){
                info!("Request from {addr}");
                index=generate_index(match addr.ip(){
                    IpAddr::V4(ip)=>ip.to_string(),
                    IpAddr::V6(ip)=>{
                        match ip.to_ipv4(){
                            Some(ip4)=>ip4.to_string(),
                            None=>ip.to_string()
                        }
                    }
                });
                index.as_bytes()
            } else if s.starts_with("get /"){
                warn!("URI {} not found, request from {addr}", unsafe{s.split_whitespace().nth(1).unwrap_unchecked()});
                NOT_FOUND
            } else {
                error!("Bad Request `{s}` from {addr}");
                BAD_REQ
            }
        };
        #[cfg(debug_assertions)]
        for line in std::str::from_utf8(response).unwrap().split("\r\n"){
            trace!("< {line}");
        }
        stream.write(response).await?;
        request.clear();
    }
}