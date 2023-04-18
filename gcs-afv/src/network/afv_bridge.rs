use std::net::SocketAddr;

use default_net::Interface;
use log::{info, error, debug};
use tokio::{sync::broadcast, net::TcpListener};

use crate::network::{AFV_COMM_PORT, scanner::{ScanBuilder, ScanCount}, socket::Socket};

use super::NetMessage;

pub struct AfvBridge{
    
}

impl AfvBridge{
    pub async fn client(tx: broadcast::Sender<NetMessage>, rx: broadcast::Sender<NetMessage>, scan_count: ScanCount){
        info!("Starting Afv bridge search using port {}", AFV_COMM_PORT);
        let scan = ScanBuilder::default().scan_count(scan_count).add_port(AFV_COMM_PORT).dispatch();
        while let Ok(stream) = scan.recv_async().await{
            debug!("Afv found at addr {}", stream.peer_addr().unwrap());
            let socket = Socket::new(stream, false);
            tokio::spawn(Self::communicate(tx.clone(), rx.subscribe(), socket));
        }
        info!("Afv bridge search with port {} completed", AFV_COMM_PORT);
    }
    pub async fn server(tx: broadcast::Sender<NetMessage>, rx: broadcast::Sender<NetMessage>, tgt_interface: Option<Interface>){
        info!("Opening afv bridge server on port {}", AFV_COMM_PORT);
        
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
                    error!("Could not get default interface. Canceling afv bridge server on port {}", AFV_COMM_PORT);
                    
                    return;
                },
            },
        };


        let ip = match interface.ipv4.first(){
            Some(ip) => {
                info!("Default ip address {:?}", ip);
                ip.addr
            },
            None => {
                error!("No default ip address found!");
                return;
            },
        };

        if let Ok(listener) = TcpListener::bind((ip, AFV_COMM_PORT)).await{
            debug!("Afv bridge server listening on {}", SocketAddr::from((ip, AFV_COMM_PORT)));
            if let Ok((stream, peer)) = listener.accept().await{
                debug!("Afv bridge as been linked to {}", peer);
                let socket = Socket::new(stream, true);
                tokio::spawn(Self::communicate(tx, rx.subscribe(), socket));
            }
        }
    }
    pub async fn direct_connect(tx: broadcast::Sender<NetMessage>, rx: broadcast::Receiver<NetMessage>, tgt: SocketAddr){
        todo!()
    }

    async fn communicate(tx: broadcast::Sender<NetMessage>, rx: broadcast::Receiver<NetMessage>, socket: Socket){
        
    }
}