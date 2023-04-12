use std::{sync::Arc, net::SocketAddr};

use async_trait::async_trait;
use rand::{thread_rng, Rng};
use tokio::{net::{TcpStream, ToSocketAddrs, TcpListener}, sync::RwLock};

use crate::{bus::{BusElement, Bus, BusUuid}, afv::AfvUuid, messages::{AfvCtlMessage, NetworkMessages}};

use super::socket::Socket;

pub struct NetworkBridge{
    uuid: BusUuid,
    bus: Bus<AfvCtlMessage>,
    afv_uuid: RwLock<Option<AfvUuid>>,
    socket: Socket,
}

impl NetworkBridge{
    pub async fn new(bus: &Bus<AfvCtlMessage>, stream: TcpStream, is_server: bool) -> Arc<NetworkBridge> {
        let socket = Socket::new(stream, is_server);
        
        let bridge = Arc::new(Self{
            uuid: thread_rng().gen(),
            bus: bus.clone(),
            afv_uuid: Default::default(),
            socket,
        });

        tokio::spawn(bridge.clone().listen());

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
                        println!("Tcp server using up {:?}", i);
                        i.addr
                    },
                    None => {
                        println!("Tcp server no ip");
                        return None;
                    },
                }
            },
            Err(e) => {
                println!("Tcp server error {}", e);
                return None;
            },
        };
        
        if let Ok(l) = TcpListener::bind((ip, port)).await{
            println!("Tcp server bound to {}", SocketAddr::from((ip, port)));
            if let Ok((s,_)) = l.accept().await{
                println!("Tcp server link {} -> {}", s.local_addr().unwrap(), s.peer_addr().unwrap());
                return Some(Self::new(bus, s, true).await);
            }
        }
        None
    }

    async fn listen(self: Arc<Self>){
        let mut data:Vec<u8> = vec![];
        
        loop{
            // The function only ends in error
            let byte = self.socket.clone().read_byte().await;
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
    }

    async fn forward(self: Arc<Self>, msg: AfvCtlMessage){
        // println!("Forwarding message {:?}", msg);
        if let Some(afv_uuid) = *self.afv_uuid.read().await{
            if let AfvCtlMessage::Network(msg) = msg.clone(){
                match msg{
                    NetworkMessages::FlirStream(uuid) => {
                        if uuid != afv_uuid{return}
                    },
                    NetworkMessages::FlirFilterLevel(uuid, _) => {
                        if uuid != afv_uuid{return}
                    },
                    NetworkMessages::FlirTargetIterations(uuid, _) => {
                        if uuid != afv_uuid{return}
                    },
                    NetworkMessages::PollFlirAngle(uuid) => {
                        if uuid != afv_uuid{return}
                    },
                    NetworkMessages::PollDistance(uuid) => {
                        if uuid != afv_uuid{return}
                    },
                    NetworkMessages::FlirAngle(uuid, _, _) => {
                        if uuid != afv_uuid{return}
                    },
                    NetworkMessages::Distance(uuid, _) => {
                        if uuid != afv_uuid{return}
                    },
                    NetworkMessages::PollFiringSolution(uuid) => {
                        if uuid != afv_uuid{return}
                    },
                    NetworkMessages::AutoTarget(uuid) => {
                        if uuid != afv_uuid{return}
                    },
                    NetworkMessages::AutoAim(uuid) => {
                        if uuid != afv_uuid{return}
                    },
                    NetworkMessages::PollAfvUuid => {},
                    NetworkMessages::AfvUuid(_) => {},
                    NetworkMessages::NalPacket(_) => {},
                }
            }
        }
        if let Ok(msg) = bincode::serialize(&msg){
            self.socket.write_data(&msg).await;
        }
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