use std::{marker::PhantomData, sync::Arc};

use afv_internal::FLIRTURRETPORT;
use tokio::{sync::Mutex, net::TcpStream, time::{sleep, Duration}};

use crate::network::{socket::Socket, scanner::ScanBuilder, Network, Local};

pub struct FlirTurret<NetType>{
    socket: Arc<Mutex<Option<Socket>>>,
    _net: PhantomData<NetType>,
}

impl FlirTurret<Local>{
    pub async fn new() -> Arc<Self> {
        let ctl = Arc::new(Self{
            _net: PhantomData,
            socket: Default::default(),
        });

        tokio::spawn(ctl.clone().initial_connection());

        ctl
    }

    async fn initial_connection(self: Arc<Self>){
        let scan = ScanBuilder::default()
        .scan_count(crate::network::scanner::ScanCount::Infinite)
        .add_port(FLIRTURRETPORT)
        .dispatch();

        let stream = match scan.recv_async().await{
            Ok(stream) => stream,
            Err(_) => return,
        };

        drop(scan);

        println!("Connected to flir turret at {}", stream.peer_addr().unwrap());

        *self.socket.lock().await = Some(Socket::new(stream, false));

        // For testing only
        loop{
            println!("Pinging flir turret");
            let msg = afv_internal::network::InternalMessage::Ping(100).to_msg().unwrap();
            let _ = self.socket.lock().await.as_ref().unwrap().clone().write_data(&msg).await;
            sleep(Duration::from_secs(1)).await;
        }
    }

}

impl FlirTurret<Network>{
    pub async fn new() -> Arc<Self> {
        Arc::new(Self{
            socket: Default::default(),
            _net: PhantomData,
        })
    }
    
}

impl<T: Send + Sync + 'static> FlirTurret<T>{
    pub async fn adjust_angle(&self, angles: (f32, f32)){
        
    }
}