use std::net::{SocketAddr, Ipv4Addr};

use afv_internal::{FLIR_TURRET_PORT, NOZZLE_TURRET_PORT};
use tokio::{sync::broadcast, time::sleep, net::TcpStream};

use crate::{network::{NetMessage, afv_bridge::AfvBridge, scanner::ScanCount}, drivers::{turret::TurretDriver, lidar::LidarDriver}};

use super::{naming::NamingOperator, flir::FlirOperator};

pub async fn launch(client: bool, direct_connect: Option<SocketAddr>){
    let (net_tx, _rx) = broadcast::channel::<NetMessage>(10000);
    if client{
        match direct_connect{
            Some(addr) => {
                tokio::spawn(AfvBridge::direct_connect(net_tx.clone(), net_tx.clone(), addr));
            },
            None => {
                tokio::spawn(AfvBridge::client(net_tx.clone(), ScanCount::Limited(3)));
            },
        }
    }
    else{
        tokio::spawn(AfvBridge::server(net_tx.clone(), None));
    }

    tokio::spawn(NamingOperator::new(net_tx.clone()));
    tokio::spawn(FlirOperator::new(net_tx.clone()));
    tokio::spawn(TurretDriver::new(net_tx.clone(), FLIR_TURRET_PORT));
    tokio::spawn(TurretDriver::new(net_tx.clone(), NOZZLE_TURRET_PORT));
    tokio::spawn(LidarDriver::new(net_tx.clone()));
    loop{
        sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

pub async fn simulate(){
    let (net_tx, _rx) = broadcast::channel::<NetMessage>(10000);
    tokio::spawn(AfvBridge::server(net_tx.clone(), None));
    tokio::spawn(NamingOperator::new(net_tx.clone()));
    tokio::spawn(FlirOperator::new(net_tx.clone()));
    tokio::spawn(TurretDriver::new(net_tx.clone(), FLIR_TURRET_PORT));
    tokio::spawn(TurretDriver::new(net_tx.clone(), NOZZLE_TURRET_PORT));
    tokio::spawn(LidarDriver::new(net_tx.clone()));
}
