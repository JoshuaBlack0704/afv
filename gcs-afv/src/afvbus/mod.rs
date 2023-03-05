use std::{sync::Arc, net::SocketAddr, marker::PhantomData};

use async_trait::async_trait;
use clap::Parser;
use rand::{thread_rng, Rng};
use tokio::{time::sleep, runtime::Handle};

use crate::{bus::{Bus, BusUuid, BusElement}, networkbus::networkbridge::NetworkBridge, GCSBRIDGEPORT, messages::{AfvCtlMessage, NetworkMessages}};

#[derive(Parser, Debug)]
pub struct AfvArgs{
    connect: Option<SocketAddr>,
}

pub type AfvUuid = u16;
pub struct Simulated;
pub struct Actuated;
pub struct Afv<SimType>{
    bus_uuid: BusUuid,
    afv_uuid: AfvUuid,
    bus: Bus<AfvCtlMessage>,
    handle: Handle,
    _sim: PhantomData<SimType>,
}

impl<T> Afv<T>{
    pub fn simulate(){
        let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not build runtime"));
        rt.block_on(async move {
            let bus = Bus::<AfvCtlMessage>::new().await;
            println!("Afv listening on port {}", GCSBRIDGEPORT);
            NetworkBridge::server(&bus, GCSBRIDGEPORT).await;
            println!("Afv connected on port {}", GCSBRIDGEPORT);
            let afv:Arc<Afv<Simulated>> = Arc::new(Afv{
                bus_uuid: thread_rng().gen(),
                afv_uuid: thread_rng().gen(),
                bus: bus.clone(),
                handle: Handle::current(),
                _sim: PhantomData,
            });

            bus.add_element(afv.clone()).await;
            
            loop{
                sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });
        
    }

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
                handle: Handle::current(),
                _sim: PhantomData,
            });

            bus.add_element(afv.clone()).await;
            
            loop{
                sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });
    }
}

#[async_trait]
impl<T: Send + Sync> BusElement<AfvCtlMessage> for Afv<T>{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        if let AfvCtlMessage::Network(msg) = msg{
            match msg{
                NetworkMessages::PollAfvUuid => {
                    // This message type is handled the same
                    tokio::spawn(
                        self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Network(NetworkMessages::AfvUuid(self.afv_uuid)))
                    );
                },
                _ => {}
            }
        }
        
    }
    fn uuid(&self) -> BusUuid{
        self.bus_uuid
    }
}