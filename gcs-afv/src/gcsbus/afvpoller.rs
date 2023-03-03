use std::sync::Arc;

use async_trait::async_trait;
use eframe::egui::Ui;
use rand::{thread_rng, Rng};
use tokio::net::TcpStream;

use crate::{AfvCtlMessage, bus::{BusElement, Bus}, GCSBRIDGEPORT};

use super::Renderable;

pub struct AfvPoller{
    uuid: u64,
    bus: Bus<AfvCtlMessage>,
}

impl AfvPoller{
    pub async fn new(bus: Bus<AfvCtlMessage>) -> Arc<AfvPoller> {
        let poller = Arc::new(
            Self{
                bus,
                uuid: thread_rng().gen::<u64>(), 
            }
        );
        
        poller
    }
}

#[async_trait]
impl BusElement<AfvCtlMessage> for AfvPoller{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        
    }
    fn uuid(&self) -> u64{
        self.uuid
    }
}

impl Renderable for AfvPoller{
    fn render(&self, ui: &mut Ui) {
        ui.label("Poller");
    }
}

