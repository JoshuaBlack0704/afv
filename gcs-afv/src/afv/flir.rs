use std::sync::Arc;

use futures::StreamExt;
use retina::{client::{SessionOptions, self, SetupOptions, PlayOptions}, codec::CodecItem::VideoFrame};
use tokio::time::Instant;
use url::Url;

use crate::{network::scanner::ScanBuilder, messages::{AfvCtlMessage, NetworkMessages, LocalMessages}};

use super::{Afv, FLIR_TIME};

impl<T> Afv<T>{
    pub (super) async fn stream_flir(self: Arc<Self>){
        // The first step is to attempt a connection to the flir
        let scan = ScanBuilder::default()
        .scan_count(crate::network::scanner::ScanCount::Infinite)
        .add_port(554)
        .dispatch();
        println!("Started flir scan");

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
        
        println!("FLIR ACTUATOR: Rtsp stream opened on {}", flir_ip);
        
        tokio::pin!(demux);

        // Now that we have a stream we can begin to pull NAL packets out

        loop{
            let frame;
            match demux.next().await{
                Some(f) => {
                    // We only care about video frames
                    if let Ok(VideoFrame(v)) = f{
                        frame = v.into_data();
                    }
                    else{
                        continue;
                    }
                },
                None => {continue;},
            }

            if Instant::now().duration_since(*self.flir_net_request.read().await).is_zero(){
                // The command instant has not passed, meaning we have charge to send a nal packet
                self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Network(NetworkMessages::NalPacket(frame.clone()))).await;
            }
            if Instant::now().duration_since(*self.flir_local_request.read().await).is_zero(){
                // The command instant has not passed, meaning we have charge to send a nal packet
                self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Local(LocalMessages::NalPacket(frame))).await;
            }
        }
    }
    pub (super) async fn flir_net_request(self: Arc<Self>){
        if let Some(i) = Instant::now().checked_add(FLIR_TIME){
            *self.flir_net_request.write().await = i;
        }
    }
    pub (super) async fn flir_local_request(self: Arc<Self>){
        if let Some(i) = Instant::now().checked_add(FLIR_TIME){
            *self.flir_local_request.write().await = i;
        }
    }
}
