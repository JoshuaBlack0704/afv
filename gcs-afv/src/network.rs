use std::{sync::Arc, net::ToSocketAddrs, io::Error};

use tokio::{net::{TcpStream, tcp::{OwnedWriteHalf, OwnedReadHalf}, TcpSocket}, sync::{Mutex, RwLock}, runtime::Runtime};
use async_trait::async_trait;

pub const GCSPORT:u32 = 60000;
pub const AFVPORT:u32 = 4040;

/// The trait that an object must implement should it wish to listen
/// to an ethernet bus
#[async_trait]
pub trait EthernetListener<M>{
    async fn notify(&self, msg: M);
}

/// The general enum that will be used for communication between the gcs and the afv
pub enum NetworkMessage{
    
}

pub enum EthernetBusError{
    SocketAddr(Error),
    NoAddr,
}

/// The ethernet system that will receive and distribute all ethernet communication
pub struct EthernetBus<M>{
    read_socket: Mutex<OwnedReadHalf>,
    write_socket: Arc<Mutex<OwnedWriteHalf>>,
    listeners: RwLock<Vec<Arc<dyn EthernetListener<M>>>>,
}

impl EthernetBus<NetworkMessage>{
    pub async fn new(tgt: &impl ToSocketAddrs) -> Result<EthernetBus<NetworkMessage>, EthernetBusError> {
        let addr;
        match tgt.to_socket_addrs(){
            Ok(mut a) => {
                if let Some(a) = a.next(){
                    addr = a;
                }
                else{
                    return Err(EthernetBusError::NoAddr);
                }
            },
            Err(e) => return Err(EthernetBusError::SocketAddr(e)),
        };
        
        printl

        
        todo!()

    }
    pub fn new_blocking(runtime: Arc<Runtime>, tgt: &impl ToSocketAddrs){
        
    }
}