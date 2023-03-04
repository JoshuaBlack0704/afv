use std::sync::Arc;

use async_trait::async_trait;
use rand::{thread_rng, Rng};
use tokio::{net::{tcp::{OwnedWriteHalf, OwnedReadHalf}, TcpStream, ToSocketAddrs, TcpListener}, sync::{RwLock, Mutex}, io::{AsyncReadExt, AsyncWriteExt}};

use crate::{bus::{BusElement, Bus}, AfvCtlMessage};

pub struct NetworkBridge{
    uuid: u64,
    bus: Bus<AfvCtlMessage>,
    afv_uuid: RwLock<Option<u64>>,
    write: Mutex<OwnedWriteHalf>,
}

impl NetworkBridge{
    pub async fn new(bus: &Bus<AfvCtlMessage>, stream: TcpStream) -> Arc<NetworkBridge> {
        let (rd, wr) = stream.into_split();
        
        let bridge = Arc::new(Self{
            uuid: thread_rng().gen::<u64>(),
            bus: bus.clone(),
            afv_uuid: Default::default(),
            write: Mutex::new(wr),
        });

        tokio::spawn(bridge.clone().listen(rd));

        bus.add_element(bridge.clone()).await;

        bridge
    }

    pub async fn client(bus: &Bus<AfvCtlMessage>, addr: &impl ToSocketAddrs) -> Option<Arc<Self>> {
        if let Ok(s) = TcpStream::connect(addr).await{
            return Option::Some(Self::new(bus, s).await);
        }
        None
    }
    pub async fn server(bus: &Bus<AfvCtlMessage>, port: u16) -> Option<Arc<Self>> {
        let ip = match default_net::get_default_interface(){
            Ok(i) => {
                match i.ipv4.first(){
                    Some(i) => {
                        i.addr
                    },
                    None => return None,
                }
            },
            Err(_) => return None,
        };
        
        if let Ok(l) = TcpListener::bind((ip, port)).await{
            if let Ok((s,_)) = l.accept().await{
                return Some(Self::new(bus, s).await);
            }
        }
        None
    }

    async fn listen(self: Arc<Self>, mut rd: OwnedReadHalf){
        let mut data:Vec<u8> = vec![];
        
        // The function only ends in error
        while let Ok(byte) = rd.read_u8().await{
            data.push(byte); 

            if let Ok(msg) = bincode::deserialize::<AfvCtlMessage>(&data){
                println!("Received message {:?}", msg);
                if let AfvCtlMessage::NetworkAfvUUID(uuid) = msg{
                    *self.afv_uuid.write().await = Some(uuid);
                }
                self.bus.send(self.uuid, msg).await;
                data.clear();
            }
        }

        // TODO: implement reconnect procedure
    }

    async fn forward(self: Arc<Self>, msg: AfvCtlMessage){
        println!("Forwarding message {:?}", msg);
        if let Ok(msg) = bincode::serialize(&msg){
            let _ = self.write.lock().await.write(&msg).await;
        }
    }

}

#[async_trait]
impl BusElement<AfvCtlMessage> for NetworkBridge{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        // We only allow network msgs through
        match msg{
            AfvCtlMessage::NetworkAfvUUID(_) => {
                tokio::spawn(self.forward(msg));
            },
            AfvCtlMessage::NetworkAfvUUIDPoll => {
                tokio::spawn(self.forward(msg));
            },
        }
    }
    fn uuid(&self) -> u64{
        self.uuid
    }
}