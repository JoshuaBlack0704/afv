use std::sync::Arc;


use eframe::egui::Ui;
use rand::{thread_rng, Rng};
use tokio::{net::TcpStream, sync::Mutex, runtime::Handle, time::{Duration, sleep}};

use crate::{bus::{Bus, BusUuid}, GCSBRIDGEPORT, network::{scanner::ScanCount, networkbridge::NetworkBridge}, messages::{AfvCtlMessage, NetworkMessages}};
use crate::network::scanner::ScanBuilder;

use super::Renderable;

pub struct BridgeFinder{
    uuid: BusUuid,
    bus: Bus<AfvCtlMessage>,
    scan: Mutex<Option<flume::Receiver<TcpStream>>>,
    handle: Handle,
    afv_poll_interval: Duration,
}

impl BridgeFinder{
    pub async fn new(bus: Bus<AfvCtlMessage>, afv_poll_interval: Duration) -> Arc<BridgeFinder> {
        let bridge_finder = Arc::new(
            Self{
                bus,
                uuid: thread_rng().gen(),
                scan: Default::default(),
                handle: Handle::current(),
                afv_poll_interval,
            }
        );

        tokio::spawn(bridge_finder.clone().poll_bridges());

        bridge_finder
    }

    pub fn process_bridges(&self){
        if let Some(rx) = &(*self.scan.blocking_lock()){
            while let Ok(stream) = rx.try_recv(){
                self.handle.block_on(async move {
                    self.bus.add_element(NetworkBridge::new(&self.bus, stream, false).await).await
                });
            }
        }
    }

    async fn poll_bridges(self: Arc<Self>){
        loop{
            self.bus.clone().send(self.uuid, AfvCtlMessage::Network(NetworkMessages::PollAfvUuid)).await;
            sleep(self.afv_poll_interval).await;
        }
    }

}

impl Renderable for BridgeFinder{
    fn render(&self, ui: &mut Ui) {
        // This is called while in the top row of the terminal
        let mut lock = self.scan.blocking_lock();
        match &(*lock){
            Some(_) => {
                if ui.button("Stop Scan")
                .on_hover_ui(|ui|{
                       ui.label("Will stop scanning for network bridges"); 
                    })
                .clicked(){
                *lock = None;
                }
            },
            None => {
                if ui.button("Start Bridge Scan")
                .on_hover_ui(|ui|{
                   ui.label("Will start scanning for network bridges"); 
                })
                .clicked(){
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

