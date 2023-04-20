use std::sync::Arc;

use eframe::egui::Ui;
use tokio::{sync::{broadcast, Mutex}, runtime::Handle};

use crate::{network::{socket::Socket, afv_bridge::AfvBridge, NetMessage, scanner::ScanCount}, ui::Renderable};

use super::{naming::NamingSystemCommunicator, flir::FlirSystemCommunicator};

pub struct AfvCommuncation{
    handle: Handle,
    tx: broadcast::Sender<NetMessage>,
    naming_system: NamingSystemCommunicator,
    flir_system: Option<FlirSystemCommunicator>,
}

impl AfvCommuncation{
    pub async fn uuid(&mut self) -> u64 {
        self.naming_system.uuid().await
    }
    pub async fn find_afvs(afvs: Arc<Mutex<Vec<AfvCommuncation>>>, scan_count: ScanCount){
        let discovered_afvs = AfvBridge::scan(scan_count);
        while let Ok(socket) = discovered_afvs.recv_async().await{
            let communication = Self::start_communication(socket).await; 
            afvs.lock().await.push(communication);
        }
    }
    async fn start_communication(socket: Socket) -> Self{
        let (tx,_rx) = broadcast::channel(10000);
        AfvBridge::start_communication(tx.clone(), socket);
        

        Self{
            tx: tx.clone(),
            naming_system: NamingSystemCommunicator::new(tx.clone()).await,
            flir_system: Default::default(),
            handle: Handle::current(),
        }
    }
}

impl Renderable for AfvCommuncation{
    fn render(&mut self, ui: &mut Ui) {
        let flir_system = self.flir_system.get_or_insert_with(||{self.handle.block_on(FlirSystemCommunicator::new(self.tx.clone(), ui))});
        flir_system.render(ui);
    }
}
