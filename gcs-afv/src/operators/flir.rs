use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;

use crate::{drivers::flir::FlirDriver, network::NetMessage};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FlirOperatorMessage{
    Settings(FlirOperatorSettings),
    SetSettings(FlirOperatorSettings),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FlirOperatorSettings{
    pub fliter_value: u32,
    
}
impl Default for FlirOperatorSettings{
    fn default() -> Self {
        Self{
            fliter_value: 200,
        }
    }
}

pub struct FlirOperator{
    driver: FlirDriver,
}

impl FlirOperator{
    pub async fn new(tx: broadcast::Sender<NetMessage>) -> FlirOperator {
        Self{
            driver: FlirDriver::new(tx.clone(), false).await,
        }
    }
}