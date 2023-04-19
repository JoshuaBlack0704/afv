use std::sync::Arc;

use log::{trace, info};
use tokio::sync::{Mutex, broadcast};

use crate::network::NetMessage;

#[derive(Clone)]
pub struct NamingSystemCommunicator{
    tx: broadcast::Sender<NetMessage>,
    uuid: Arc<Mutex<u64>>,
}

impl NamingSystemCommunicator{
    pub async fn new(tx: broadcast::Sender<NetMessage>) -> Self{
        let comm = Self{
            tx,
            uuid: Arc::new(Mutex::new(0)),
        };

        tokio::spawn(comm.clone().start());
        
        comm

    }
    async fn start(self){
        info!("Naming communicator system started");
        let mut rx = self.tx.subscribe();

        loop{
            let msg = match rx.recv().await{
                Ok(msg) => msg,
                Err(_) => continue,
            };

            if let NetMessage::NamingOperator(msg) = msg{
                trace!("Naming operator received uuid {}", msg.id);
               *self.uuid.lock().await = msg.id; 
            }
        }
    }
    pub async fn uuid(&self) -> u64 {
        *self.uuid.lock().await
    }
    
}