use std::{sync::Arc, net::ToSocketAddrs, io::Error, mem::size_of, fmt::Debug};

use serde::{Deserialize, Serialize};
use tokio::{net::{TcpStream, tcp::{OwnedWriteHalf, OwnedReadHalf}}, sync::{Mutex, RwLock}, runtime::Runtime, time::{Duration, sleep}};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use async_trait::async_trait;

pub const GCSPORT:u32 = 60000;
pub const AFVPORT:u32 = 4040;

/// The trait that an object must implement should it wish to listen
/// to an ethernet bus
#[async_trait]
pub trait EthernetListener<M>: Send + Sync{
    async fn notify(self: Arc<Self>, msg: M);
}

/// The general enum that will be used for communication between the gcs and the afv
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage{
    Test,
}

#[derive(Debug)]
pub enum EthernetBusError{
    SocketAddr(Error),
    NoAddr,
    CouldNotConnect(Error)
}

/// The ethernet system that will receive and distribute all ethernet communication
pub struct EthernetBus<M>{
    read_socket: Mutex<OwnedReadHalf>,
    write_socket: Arc<Mutex<OwnedWriteHalf>>,
    listeners: RwLock<Vec<Arc<dyn EthernetListener<M>>>>,
}

pub struct NetworkLogger{
    
}

#[async_trait]
impl EthernetListener<NetworkMessage> for NetworkLogger{
    async fn notify(self: Arc<Self>, msg: NetworkMessage){
        println!("Network traffic: {:?}", msg);
    }
}

impl NetworkLogger{
    pub async fn new(bus: &EthernetBus<NetworkMessage>){
        let log = Arc::new(Self{});
        bus.add_listener(log).await;
    }
}

impl<M> EthernetBus<M>{
    pub async fn add_listener(&self, listener: Arc<dyn EthernetListener<M>>){
        let mut listeners = self.listeners.write().await;
        listeners.push(listener);
    }
    pub fn add_listener_blocking(&self, listener: Arc<dyn EthernetListener<M>>){
        let mut listeners = self.listeners.blocking_write();
        listeners.push(listener);
    }
}

impl EthernetBus<NetworkMessage>{
    pub async fn new(tgt: &impl ToSocketAddrs) -> Result<Arc<EthernetBus<NetworkMessage>>, EthernetBusError> {
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
        
        let sock = match TcpStream::connect(addr).await{
            Ok(s) => {
               s 
            },
            Err(e) => return Err(EthernetBusError::SocketAddr(e)),
        };

        let (rd, wr) = sock.into_split();
        let rd = Mutex::new(rd);
        let wr = Arc::new(Mutex::new(wr));

        let ethernet = Arc::new(Self{
            read_socket: rd,
            write_socket: wr,
            listeners: RwLock::new(vec![]),
        });

        tokio::spawn(ethernet.clone().listen());

        Ok(ethernet)

    }
    pub fn new_blocking(runtime: Arc<Runtime>, tgt: &impl ToSocketAddrs) -> Result<Arc<EthernetBus<NetworkMessage>>, EthernetBusError> {
        runtime.block_on(Self::new(tgt))
    }

    async fn listen(self: Arc<Self>){
        println!("Tcp streaming on {}", self.read_socket.lock().await.local_addr().expect("No addr for socket"));
        let sleep_time = Duration::from_secs(1);
        let mut data = Vec::with_capacity(size_of::<NetworkMessage>());
            
        while Arc::strong_count(&self) > 1{
            let preread_length = data.len();
            tokio::select!{
                _ = sleep(sleep_time) => {println!("Timeout");continue;}
                _ = self.process_msg(&mut data) => {}
            }
            let postread_length = data.len();
            if preread_length == postread_length{
                break;
            }
            println!("Processing message: {:?}", data);

            let msg:NetworkMessage = match bincode::deserialize::<NetworkMessage>(&data){
                Ok(a) => a,
                Err(_) => {continue;},
            };

            for listener in self.listeners.read().await.iter(){
                tokio::spawn(listener.clone().notify(msg.clone()));
            }

            data.clear();
            
        }

        println!("Tcp stream on {} closing", self.read_socket.lock().await.local_addr().expect("No addr for socket"));
    }

    async fn send(&self, msg: NetworkMessage){
        let msg = bincode::serialize(&msg).expect("Could not serialize msg");
        let mut write = self.write_socket.lock().await;
        if let Ok(n) = write.write_all(&msg).await{
            let _ = write.flush().await;
        } 
    }

    async fn process_msg(&self, data: &mut Vec<u8>){
        let mut read = self.read_socket.lock().await;
        if let Ok(byte) = read.read_u8().await{
            data.push(byte);
        }
    }
}