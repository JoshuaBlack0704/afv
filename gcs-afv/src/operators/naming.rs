use rand::{thread_rng, Rng};
use serde::{Serialize, Deserialize};
use tokio::{sync::broadcast, time::sleep};

use crate::network::NetMessage;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NamingOperatorMessage{
    pub id: u64,
}

pub struct NamingOperator{
    
}

impl NamingOperator{
    pub async fn new(tx: broadcast::Sender<NetMessage>){
        let uuid = thread_rng().gen();

        loop{
            sleep(tokio::time::Duration::from_secs(3)).await;
            let _ = tx.send(NetMessage::NamingOperator(NamingOperatorMessage{id: uuid}));
        }
    }
    
}

