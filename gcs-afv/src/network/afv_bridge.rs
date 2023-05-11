use std::net::SocketAddr;

use default_net::Interface;
use log::{info, error, debug};
use tokio::{sync::{broadcast, watch}, net::TcpListener};

use crate::{network::{AFV_COMM_PORT, scanner::{ScanBuilder, ScanCount}, socket::Socket}, operators::naming::NamingOperatorMessage};

use super::NetMessage;

/// The AfvBridge is a bus member that is designed to transparently broadcast messages from 
/// one bus to another. Essentially making it so that two computers share a logical bus
/// This functionality is the foundation for the publish-filter architecture of the code base.
/// The AfvBridge is the struct that finds and connects to afvs and ground stations.
pub struct AfvBridge{
}

impl AfvBridge{
    /// A wrapper around a ScanBuilder that auto fills some data and returns that successful Sockets
    pub fn scan(scan_count: ScanCount) -> flume::Receiver<Socket>{
        let (tx, rx) = flume::unbounded();
        // Starting the scan
        let scan = ScanBuilder::default().scan_count(scan_count).add_port(AFV_COMM_PORT).dispatch();
        tokio::spawn(async move{
            info!("Starting Afv scan search using port {}", AFV_COMM_PORT);
            while let Ok(stream) = scan.recv_async().await{
                info!("Afv found at addr {}", stream.peer_addr().unwrap());
                // Wrapping in Socket struct
                let socket = Socket::new(stream, false);
                if let Err(_) = tx.send_async(socket).await{break;}
            }
            info!("Afv scan with port {} completed", AFV_COMM_PORT);
        });
        rx
    }
    /// A wrapper around a ScanBuilder that will search for tcp servers on the AFV_COMM_PORT tcp port
    /// Note, since we are using async architecture, what happens is for every successful connection 
    /// we spawn a listen task with a copy of the bus channel transmitter that will transparently receive and
    /// and send data from/to the bus. That is why this method does not need to return anything
    pub async fn client(tx: broadcast::Sender<NetMessage>, scan_count: ScanCount){
        info!("Starting Afv server search using port {}", AFV_COMM_PORT);
        let scan = ScanBuilder::default().scan_count(scan_count).add_port(AFV_COMM_PORT).dispatch();
        while let Ok(stream) = scan.recv_async().await{
            info!("Afv server found at addr {}", stream.peer_addr().unwrap());
            let socket = Socket::new(stream, false);
            Self::start_communication(tx.clone(), socket);
        }
        info!("Afv server search with port {} completed", AFV_COMM_PORT);
    }
    /// This will spawn a SINGLE AfvBridge that will wait for a connection to be made. Then it will spawn the listen task 
    pub async fn server(tx: broadcast::Sender<NetMessage>, tgt_interface: Option<Interface>){
        info!("Opening afv bridge server on port {}", AFV_COMM_PORT);
        
        let interface = match tgt_interface {
            Some(i) => {
                    match &i.friendly_name{
                        Some(n) => {
                            debug!("Default interface is {}", n);
                        }
                        None => {
                            debug!("Default interface is {}", i.name);
                        }
                    }
                i
                
            },
            None => match default_net::get_default_interface(){
                Ok(i) => {
                    match &i.friendly_name{
                        Some(n) => {
                            debug!("Default interface is {}", n);
                        }
                        None => {
                            debug!("Default interface is {}", i.name);
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
                debug!("Default ip address {:?}", ip);
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
                info!("Afv bridge as been linked to {}", peer);
                let socket = Socket::new(stream, true);
                Self::start_communication(tx, socket);
            }
        }
    }
    #[allow(unused)]
    pub async fn direct_connect(bridge_tx: broadcast::Sender<NetMessage>, bus_tx: broadcast::Sender<NetMessage>, tgt: SocketAddr){
        todo!()
    }
    /// A helper function to start the neccesary tasks to make a functional bridge system
    pub fn start_communication(tx: broadcast::Sender<NetMessage>, socket: Socket){
        let (d_tx, d_rx) = watch::channel(NetMessage::NamingOperator(NamingOperatorMessage{id:  0}));

        tokio::spawn(Self::forward(tx.subscribe(), d_rx, socket.clone()));
        tokio::spawn(Self::listen(tx, d_tx, socket.clone()));
    }
    /// The main task that will put network tasks on the local bus
    async fn listen(tx: broadcast::Sender<NetMessage>, duplicates: watch::Sender<NetMessage>, socket: Socket){
        let mut data = vec![];

        loop{
            let byte = socket.read_byte().await;
            data.push(byte);

            let msg = match bincode::deserialize::<NetMessage>(&data){
                Ok(msg) => msg,
                Err(_) => continue,
            };

            debug!("Afv bridge traffic {}<-{}: {:?}", socket.local_addr(), socket.peer_addr(), msg);

            let _ = duplicates.send(msg.clone());
            let _ = tx.send(msg);

            data.clear();
        }
    }
    /// The main task that will take local bus messages and broadcast over the network
    async fn forward(mut rx: broadcast::Receiver<NetMessage>, mut duplicates: watch::Receiver<NetMessage>, socket: Socket){
       loop{
            let msg = match rx.recv().await{
                Ok(msg) => msg,
                Err(e) => {
                    error!("Failed to pull message from bus: {}", e);
                    continue;
                },
            };
            if msg == *duplicates.borrow_and_update(){ continue; }
            let data = match bincode::serialize(&msg){
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to serialize message {:?} with error {}", msg, e);
                    continue;
                },
            };

            debug!("Afv bridge traffic {}->{}: {:?}", socket.local_addr(), socket.peer_addr(), msg);

            socket.write_data(&data).await;
        } 
    }
}