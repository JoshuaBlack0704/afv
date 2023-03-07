use std::sync::Arc;

use async_trait::async_trait;
use rand::{thread_rng, Rng};
use tokio::{net::{tcp::{OwnedWriteHalf, OwnedReadHalf}, TcpStream, ToSocketAddrs, TcpListener}, sync::{RwLock, Mutex}, io::{AsyncReadExt, AsyncWriteExt}};

use crate::{bus::{BusElement, Bus, BusUuid}, afvbus::AfvUuid, messages::{AfvCtlMessage, NetworkMessages}};

pub struct NetworkBridge{
    server: bool,
    uuid: BusUuid,
    bus: Bus<AfvCtlMessage>,
    afv_uuid: RwLock<Option<AfvUuid>>,
    write: Mutex<OwnedWriteHalf>,
}

impl NetworkBridge{
    pub async fn new(bus: &Bus<AfvCtlMessage>, stream: TcpStream, is_server: bool) -> Arc<NetworkBridge> {
        let (rd, wr) = stream.into_split();
        
        let bridge = Arc::new(Self{
            uuid: thread_rng().gen(),
            bus: bus.clone(),
            afv_uuid: Default::default(),
            write: Mutex::new(wr),
            server: is_server,
        });

        tokio::spawn(bridge.clone().listen(rd));

        bus.add_element(bridge.clone()).await;

        bridge
    }

    pub async fn client(bus: &Bus<AfvCtlMessage>, addr: &impl ToSocketAddrs) -> Option<Arc<Self>> {
        if let Ok(s) = TcpStream::connect(addr).await{
            return Option::Some(Self::new(bus, s, false).await);
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
                return Some(Self::new(bus, s, true).await);
            }
        }
        None
    }

    async fn listen(self: Arc<Self>, mut rd: OwnedReadHalf){
        let mut data:Vec<u8> = vec![];
        
        loop{
            // The function only ends in error
            while let Ok(byte) = rd.read_u8().await{
                data.push(byte); 

                if let Ok(msg) = bincode::deserialize::<AfvCtlMessage>(&data){
                    // println!("Received message {:?}", msg);
                    if let AfvCtlMessage::Network(NetworkMessages::AfvUuid(uuid)) = msg{
                        *self.afv_uuid.write().await = Some(uuid);
                    }
                    self.bus.clone().send(self.uuid, msg).await;
                    data.clear();
                }
            }
        

            // TODO: implement reconnect procedure

            if self.server{
                rd = self.reconnect_server().await;
            }
            else{
                rd = self.reconnect_client().await;
            }
        }
    }

    async fn forward(self: Arc<Self>, msg: AfvCtlMessage){
        // println!("Forwarding message {:?}", msg);
        if let Ok(msg) = bincode::serialize(&msg){
            let _ = self.write.lock().await.write(&msg).await;
        }
    }

    async fn reconnect_server(&self) -> OwnedReadHalf{
        todo!()
    }
    async fn reconnect_client(&self) -> OwnedReadHalf{
        todo!()
    }

}

#[async_trait]
impl BusElement<AfvCtlMessage> for NetworkBridge{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        // We only allow network msgs through
        if let AfvCtlMessage::Network(_) = msg{
            tokio::spawn(self.forward(msg));
        }
    }
    fn uuid(&self) -> BusUuid{
        self.uuid
    }
}