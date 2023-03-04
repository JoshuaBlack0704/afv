use std::{sync::Arc, net::SocketAddr};

use clap::Parser;
use rand::{thread_rng, Rng};
use tokio::{net::ToSocketAddrs, time::sleep};

use crate::{bus::Bus, AfvCtlMessage, networkbus::networkbridge::NetworkBridge, GCSBRIDGEPORT};

use self::pollresponder::PollResponder;

#[derive(Parser, Debug)]
pub struct AfvArgs{
    connect: Option<SocketAddr>,
}

pub struct Afv;

mod pollresponder;


impl Afv{
    fn client(addr: &impl ToSocketAddrs){
        let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not build runtime"));
        rt.block_on(async move {
            let afv_uuid = thread_rng().gen::<u64>();
            let bus = Bus::<AfvCtlMessage>::new().await;
            PollResponder::new(bus.clone(), afv_uuid).await;
            println!("Afv connecting");
            NetworkBridge::client(&bus, addr).await;
            println!("Afv connected");
            
            loop{
                sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });
    }
    fn server(){
        let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not build runtime"));
        rt.block_on(async move {
            let afv_uuid = thread_rng().gen::<u64>();
            let bus = Bus::<AfvCtlMessage>::new().await;
            PollResponder::new(bus.clone(), afv_uuid).await;
            println!("Afv listening on port {}", GCSBRIDGEPORT);
            NetworkBridge::server(&bus, GCSBRIDGEPORT).await;
            println!("Afv connected on port {}", GCSBRIDGEPORT);
            
            loop{
                sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });
    }
    pub fn launch(){
        let args = AfvArgs::parse();
        match args.connect{
            Some(addr) => {
                Self::client(&addr);
            },
            None => {
                Self::server()
            },
        }
    }
}