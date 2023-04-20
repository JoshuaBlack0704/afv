use std::sync::Arc;

use image::DynamicImage;
use serde::{Serialize, Deserialize};
use tokio::{sync::{broadcast, watch}, time::sleep};

use crate::{drivers::flir::FlirDriver, network::NetMessage};

pub const BROADCAST_SETTINGS_INTERVAL: u64 = 5;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FlirOperatorMessage{
    Settings(FlirOperatorSettings),
    SetSettings(FlirOperatorSettings),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FlirOperatorSettings{
    pub fliter_value: u32,
    
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct FlirAnalysis{
    lower_centroid: [f32; 2],
    upper_centroid: [f32; 2],
}

impl Default for FlirOperatorSettings{
    fn default() -> Self {
        Self{
            fliter_value: 200,
        }
    }
}

#[derive(Clone)]
pub struct FlirOperator{
    net_tx: broadcast::Sender<NetMessage>,
    settings_watch: Arc<watch::Sender<FlirOperatorSettings>>,
    _driver: FlirDriver,
}

impl FlirOperator{
    pub async fn new(net_tx: broadcast::Sender<NetMessage>) -> FlirOperator {
        let settings_watch = watch::channel(FlirOperatorSettings{fliter_value: 100});

        
        let operator = Self{
            _driver: FlirDriver::new(net_tx.clone(), true).await,
            net_tx,
            settings_watch: Arc::new(settings_watch.0),
        };

        tokio::spawn(operator.clone().settings_update_task());
        tokio::spawn(operator.clone().settings_broadcast_task());
        operator
    }

    async fn settings_update_task(self){
        let mut net_rx = self.net_tx.subscribe();
        loop{
            if let Ok(NetMessage::FlirOperator(FlirOperatorMessage::SetSettings(settings))) = net_rx.recv().await{
                let _ = self.settings_watch.send(settings);
            }
        }
    }
    async fn settings_broadcast_task(self){
        let mut settings_rx = self.settings_watch.subscribe();
        loop {
            let _ = self.net_tx.send(NetMessage::FlirOperator(FlirOperatorMessage::Settings(settings_rx.borrow_and_update().clone())));
            sleep(tokio::time::Duration::from_secs(BROADCAST_SETTINGS_INTERVAL)).await;
        }
    }
    pub fn analyze_image(image: DynamicImage){}

    
}