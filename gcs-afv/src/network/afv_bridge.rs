use std::net::SocketAddr;

use default_net::Interface;
use log::{info, error};
use tokio::sync::broadcast;

use crate::network::{AFV_COMM_PORT, scanner::{ScanBuilder, ScanCount}, socket::Socket};

use super::NetMessage;

pub struct AfvBridge{
    
}

impl AfvBridge{
    pub async fn client(tx: broadcast::Sender<NetMessage>, rx: broadcast::Receiver<NetMessage>, scan_count: ScanCount){
        info!("Starting Afv bridge search using port {}", AFV_COMM_PORT);
        let scan = ScanBuilder::default().scan_count(scan_count).add_port(AFV_COMM_PORT).dispatch();
        while let Ok(stream) = scan.recv_async().await{
            let socket = Socket::new(stream, false);
        }
        info!("Afv bridge search with port {} completed", AFV_COMM_PORT);
    }
    pub async fn server(tx: broadcast::Sender<NetMessage>, rx: broadcast::Receiver<NetMessage>, tgt_interface: Option<Interface>){
        let interface = match tgt_interface {
            Some(i) => {
                    match &i.friendly_name{
                        Some(n) => {
                            info!("Default interface is {}", n);
                        }
                        None => {
                            info!("Default interface is {}", i.name);
                        }
                    }
                i
                
            },
            None => match default_net::get_default_interface(){
                Ok(i) => {
                    match &i.friendly_name{
                        Some(n) => {
                            info!("Default interface is {}", n);
                        }
                        None => {
                            info!("Default interface is {}", i.name);
                        }
                    }

                    i
                },
                Err(_) => {
                    error!("Could not get default interface, canceling");
                    
                    return;
                },
            },
        };
    }
    pub async fn direct_connect(tx: broadcast::Sender<NetMessage>, rx: broadcast::Receiver<NetMessage>, tgt: SocketAddr){
        todo!()
    }

    async fn communicate(tx: broadcast::Sender<NetMessage>, rx: broadcast::Receiver<NetMessage>, socket: Socket){
        
    }
}