use futures::StreamExt;
use log::{info, debug};
use retina::{client::{SessionOptions, self, SetupOptions, PlayOptions}, codec::CodecItem};
use serde::{Serialize, Deserialize};
use tokio::{sync::broadcast, time::Instant};
use url::Url;

use crate::network::{NetMessage, scanner::{ScanBuilder, ScanCount}};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FlirDriverMessage{
    OpenIrStream,
    OpenVisualStream,
    #[serde(with = "serde_bytes")]
    NalPacket(Vec<u8>),
}

#[derive(Clone)]
pub struct FlirDriver{
    ir_nal_stream: broadcast::Sender<Vec<u8>>,
    visual_nal_stream: broadcast::Sender<Vec<u8>>,
}

impl FlirDriver{
    pub async fn new(net_tx: broadcast::Sender<NetMessage>, visual_stream: bool) -> FlirDriver {
        let driver = Self{
            ir_nal_stream: broadcast::channel(100).0,
            visual_nal_stream: broadcast::channel(100).0,
        };

        tokio::spawn(driver.clone().ir_stream());
        if visual_stream{
            tokio::spawn(driver.clone().visual_stream());
        }

        tokio::spawn(driver.clone().network_ir_stream(net_tx.clone()));
        tokio::spawn(driver.clone().network_visual_stream(net_tx.clone()));

        driver
        
    }

    async fn ir_stream(self){
        // The first step is to attempt a connection to the flir
        let scan = ScanBuilder::default()
        .scan_count(ScanCount::Infinite)
        .add_port(554)
        .dispatch();
        info!("Started flir if rtsp scan");

        // We will not go further until we have found a flir
        let flir_ip = match scan.recv_async().await{
            Ok(flir) => {
                match flir.peer_addr(){
                    Ok(ip) => ip,
                    Err(_) => return,
                }
            },
            Err(_) => return,
        };

        // Stop the scan
        drop(scan);

        // Now that we have found a flir we can start the stream
        let url = match Url::parse(&format!("rtsp://:@{}:554/avc", flir_ip.ip())){
            Ok(u) => {
               u 
            },
            Err(_) => return,
        };

        
        let mut options = SessionOptions::default();
        options = options.user_agent(String::from("demo"));

        let mut session = match client::Session::describe(url, options).await {
            Ok(s) => s,
            Err(_) => return,
        };

        let options = SetupOptions::default();
        if let Err(_) = session.setup(0, options).await {
            return;
        }

        let options = PlayOptions::default();
        let play = match session.play(options).await {
            Ok(p) => p,
            Err(_) => return,
        };

        let demux = match play.demuxed() {
            Ok(d) => d,
            Err(_) => return,
        };
        
        info!("Flir ir stream open with peer {}", flir_ip);
        
        tokio::pin!(demux);

        // Now that we have a stream we can begin to pull NAL packets out

        loop{
            let frame;
            match demux.next().await{
                Some(f) => {
                    // We only care about video frames
                    if let Ok(CodecItem::VideoFrame(v)) = f{
                        frame = v.into_data();
                    }
                    else{
                        continue;
                    }
                },
                None => {continue;},
            }
            let _ = self.ir_nal_stream.send(frame);
        }        
    }
    async fn visual_stream(self){
        // The first step is to attempt a connection to the flir
        let scan = ScanBuilder::default()
        .scan_count(ScanCount::Infinite)
        .add_port(554)
        .dispatch();
        info!("Started flir visual rtsp scan");

        // We will not go further until we have found a flir
        let flir_ip = match scan.recv_async().await{
            Ok(flir) => {
                match flir.peer_addr(){
                    Ok(ip) => ip,
                    Err(_) => return,
                }
            },
            Err(_) => return,
        };

        // Stop the scan
        drop(scan);

        // Now that we have found a flir we can start the stream
        let url = match Url::parse(&format!("rtsp://:@{}:554/avc/ch1", flir_ip.ip())){
            Ok(u) => {
               u 
            },
            Err(_) => return,
        };

        
        let mut options = SessionOptions::default();
        options = options.user_agent(String::from("demo"));

        let mut session = match client::Session::describe(url, options).await {
            Ok(s) => s,
            Err(_) => return,
        };

        let options = SetupOptions::default();
        if let Err(_) = session.setup(0, options).await {
            return;
        }

        let options = PlayOptions::default();
        let play = match session.play(options).await {
            Ok(p) => p,
            Err(_) => return,
        };

        let demux = match play.demuxed() {
            Ok(d) => d,
            Err(_) => return,
        };
        
        info!("Flir visual stream open with peer {}", flir_ip);
        
        tokio::pin!(demux);

        // Now that we have a stream we can begin to pull NAL packets out

        loop{
            let frame;
            match demux.next().await{
                Some(f) => {
                    // We only care about video frames
                    if let Ok(CodecItem::VideoFrame(v)) = f{
                        frame = v.into_data();
                    }
                    else{
                        continue;
                    }
                },
                None => {continue;},
            }
            let _ = self.visual_nal_stream.send(frame);
        }        
    }
    async fn network_ir_stream(self, tx: broadcast::Sender<NetMessage>){
        let mut net_rx = tx.subscribe();
        let mut last_poll = Instant::now();

        loop{
            if let Ok(NetMessage::FlirDriver(FlirDriverMessage::OpenIrStream)) = net_rx.recv().await{
                last_poll = Instant::now();
            }

            if Instant::now().duration_since(last_poll) < tokio::time::Duration::from_secs(3){
                debug!("Streaming ir video");
                let mut stream_rx = self.ir_nal_stream.subscribe();
                while Instant::now().duration_since(last_poll) < tokio::time::Duration::from_secs(3){
                    if let Ok(nal) = stream_rx.recv().await{
                        let _ = tx.send(NetMessage::FlirDriver(FlirDriverMessage::NalPacket(nal)));
                    }
                }
            }
        }
    }
    async fn network_visual_stream(self, tx: broadcast::Sender<NetMessage>){
        let mut net_rx = tx.subscribe();
        let mut last_poll = Instant::now();

        loop{
            if let Ok(NetMessage::FlirDriver(FlirDriverMessage::OpenVisualStream)) = net_rx.recv().await{
                last_poll = Instant::now();
            }

            if Instant::now().duration_since(last_poll) < tokio::time::Duration::from_secs(3){
                debug!("Streaming visual video");
                let mut stream_rx = self.visual_nal_stream.subscribe();
                while Instant::now().duration_since(last_poll) < tokio::time::Duration::from_secs(3){
                    if let Ok(nal) = stream_rx.recv().await{
                        let _ = tx.send(NetMessage::FlirDriver(FlirDriverMessage::NalPacket(nal)));
                    }
                }
            }
        }
    }
}