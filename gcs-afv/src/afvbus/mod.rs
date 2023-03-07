use std::{sync::Arc, net::SocketAddr, marker::PhantomData};

use async_trait::async_trait;
use clap::Parser;

use rand::{thread_rng, Rng};

use tokio::{time::{sleep, Instant, Duration}, runtime::Handle, sync::RwLock};


use crate::{bus::{Bus, BusUuid, BusElement}, networkbus::networkbridge::NetworkBridge, GCSBRIDGEPORT, messages::{AfvCtlMessage, NetworkMessages, LocalMessages}};

mod flir;

#[derive(Parser, Debug)]
pub struct AfvArgs{
    connect: Option<SocketAddr>,
}

pub type AfvUuid = u16;
pub const FLIR_TIME: Duration = Duration::from_secs(3);
pub struct Simulated;
pub struct Actuated;
// pub const MINSTREAMINSTANTDIFF = 
pub struct Afv<SimType>{
    // General fields
    bus_uuid: BusUuid,
    afv_uuid: AfvUuid,
    bus: Bus<AfvCtlMessage>,
    _handle: Handle,
    _sim: PhantomData<SimType>,

    // Flir fields
    flir_net_request: RwLock<Instant>,
    flir_local_request: RwLock<Instant>,
}

impl Afv<Simulated>{
    pub async fn simulate(){
        let bus = Bus::<AfvCtlMessage>::new().await;
        println!("Afv listening on port {}", GCSBRIDGEPORT);
        NetworkBridge::server(&bus, GCSBRIDGEPORT).await;
        println!("Afv connected on port {}", GCSBRIDGEPORT);
        let afv:Arc<Afv<Simulated>> = Arc::new(Afv{
            bus_uuid: thread_rng().gen(),
            afv_uuid: thread_rng().gen(),
            bus: bus.clone(),
            _handle: Handle::current(),
            _sim: PhantomData,
            flir_net_request: RwLock::new(Instant::now()),
            flir_local_request: RwLock::new(Instant::now()),
        });
        
        tokio::spawn(afv.clone().stream_flir());

        bus.add_element(afv.clone()).await;
        
        loop{
            sleep(Duration::from_secs(10)).await;
        }
        
    }
}

impl Afv<Actuated>{
    pub fn actuate(){
        let args = AfvArgs::parse();
        let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not build runtime"));
        rt.block_on(async move {
            let bus = Bus::<AfvCtlMessage>::new().await;
            match args.connect{
                Some(addr) => {
                    println!("Afv connecting");
                    NetworkBridge::client(&bus, &addr).await;
                    println!("Afv connected");
                },
                None => {
                    println!("Afv listening on port {}", GCSBRIDGEPORT);
                    NetworkBridge::server(&bus, GCSBRIDGEPORT).await;
                    println!("Afv connected on port {}", GCSBRIDGEPORT);
                },
            }
            let afv:Arc<Afv<Simulated>> = Arc::new(Afv{
                bus_uuid: thread_rng().gen(),
                afv_uuid: thread_rng().gen(),
                bus: bus.clone(),
                _handle: Handle::current(),
                _sim: PhantomData,
                flir_net_request: RwLock::new(Instant::now()),
                flir_local_request: RwLock::new(Instant::now()),
            });

            tokio::spawn(afv.clone().stream_flir());

            bus.add_element(afv.clone()).await;
            
            loop{
                sleep(Duration::from_secs(10)).await;
            }
        });
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> BusElement<AfvCtlMessage> for Afv<T>{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        if let AfvCtlMessage::Network(msg) = msg{
            match msg{
                NetworkMessages::PollAfvUuid => {
                    // This message type is handled the same
                    tokio::spawn(
                        self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Network(NetworkMessages::AfvUuid(self.afv_uuid)))
                    );
                },
                NetworkMessages::FlirStream(afv_uuid) => {
                    if self.afv_uuid != afv_uuid{return;}
                    tokio::spawn(self.clone().flir_net_request());
                },
                _ => {}
            }
            return
        }

        if let AfvCtlMessage::Local(msg) = msg{
            match msg{
                LocalMessages::FlirStream(afv_uuid) => {
                    if self.afv_uuid != afv_uuid{return;}
                    tokio::spawn(self.clone().flir_local_request());
                }
                _ => {}
            }
            return;
        }
        
    }
    fn uuid(&self) -> BusUuid{
        self.bus_uuid
    }
}