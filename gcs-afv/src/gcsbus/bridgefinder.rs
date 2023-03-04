use std::sync::Arc;

use async_trait::async_trait;
use eframe::egui::Ui;
use rand::{thread_rng, Rng};
use tokio::{net::TcpStream, sync::Mutex, runtime::Handle, time::{Duration, sleep}};

use crate::{AfvCtlMessage, bus::{BusElement, Bus}, GCSBRIDGEPORT, networkbus::{scanner::ScanCount, networkbridge::NetworkBridge}};
use crate::networkbus::scanner::ScanBuilder;

use super::Renderable;

pub struct BridgeFinder{
    uuid: u64,
    bus: Bus<AfvCtlMessage>,
    scan: Mutex<Option<flume::Receiver<TcpStream>>>,
    handle: Handle,
    afv_poll_interval: Duration,
}

impl BridgeFinder{
    pub async fn new(bus: Bus<AfvCtlMessage>, afv_poll_interval: Duration) -> Arc<BridgeFinder> {
        let poller = Arc::new(
            Self{
                bus,
                uuid: thread_rng().gen::<u64>(),
                scan: Default::default(),
                handle: Handle::current(),
                afv_poll_interval,
            }
        );

        tokio::spawn(poller.clone().poll_bridges());

        poller
    }

    pub fn process_bridges(&self){
        if let Some(rx) = &(*self.scan.blocking_lock()){
            while let Ok(stream) = rx.try_recv(){
                self.handle.block_on(async move {
                    self.bus.add_element(NetworkBridge::new(&self.bus, stream).await).await
                });
            }
        }
    }

    async fn poll_bridges(self: Arc<Self>){
        loop{
            self.bus.send(self.uuid, AfvCtlMessage::NetworkAfvUUIDPoll).await;
            sleep(self.afv_poll_interval).await;
        }
    }

}

#[async_trait]
impl BusElement<AfvCtlMessage> for BridgeFinder{
    async fn recieve(self: Arc<Self>, _msg: AfvCtlMessage){
        
    }
    fn uuid(&self) -> u64{
        self.uuid
    }
}

impl Renderable for BridgeFinder{
    fn render(&self, ui: &mut Ui) {
        let mut lock = self.scan.blocking_lock();
        match &(*lock){
            Some(_) => {
                if ui.button("Stop scan").clicked(){
                    *lock = None;
                }
            },
            None => {
                if ui.button("Start Scan").clicked(){
                    *lock = Some(
                        self.handle.block_on(async move {ScanBuilder::default()
                        .scan_count(ScanCount::Infinite)
                        .add_port(GCSBRIDGEPORT)
                        .dispatch()})
                    );
                }
            },
        }
    }
}

