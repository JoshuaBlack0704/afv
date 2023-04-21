use std::net::SocketAddr;

use afv_internal::{FLIR_TURRET_PORT, NOZZLE_TURRET_PORT};
use tokio::{sync::broadcast, time::sleep};

use crate::{network::{NetMessage, afv_bridge::AfvBridge, scanner::ScanCount}, drivers::turret::TurretDriver};

use super::{naming::NamingOperator, flir::FlirOperator};

pub async fn launch(client: bool, direct_connect: Option<SocketAddr>){
    let (tx, _rx) = broadcast::channel::<NetMessage>(10000);
    if client{
        match direct_connect{
            Some(addr) => {
                tokio::spawn(AfvBridge::direct_connect(tx.clone(), tx.clone(), addr));
            },
            None => {
                tokio::spawn(AfvBridge::client(tx.clone(), ScanCount::Limited(3)));
            },
        }
    }
    else{
        tokio::spawn(AfvBridge::server(tx.clone(), None));
    }

    tokio::spawn(NamingOperator::new(tx.clone()));
    tokio::spawn(FlirOperator::new(tx.clone()));
    tokio::spawn(TurretDriver::new(tx.clone(), FLIR_TURRET_PORT));
    tokio::spawn(TurretDriver::new(tx.clone(), NOZZLE_TURRET_PORT));
    loop{
        sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

pub async fn simulate(){
    let (tx, _rx) = broadcast::channel::<NetMessage>(10000);
    tokio::spawn(AfvBridge::server(tx.clone(), None));
    tokio::spawn(NamingOperator::new(tx.clone()));
    tokio::spawn(FlirOperator::new(tx.clone()));
    tokio::spawn(TurretDriver::new(tx.clone(), FLIR_TURRET_PORT));
    tokio::spawn(TurretDriver::new(tx.clone(), NOZZLE_TURRET_PORT));
}
