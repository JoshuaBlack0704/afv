use std::{sync::Arc, net::Ipv4Addr};

use async_trait::async_trait;
use futures::StreamExt;
use image::DynamicImage;
use retina::client::{Session, State, SessionOptions, self, SetupOptions, PlayOptions};
use serde::{Serialize, Deserialize};
use tokio::{runtime::Runtime, net::TcpStream, sync::Mutex, time::{Duration, sleep}};
use url::Url;

use crate::scanner::{Scanner, ScannerStreamHandler};

pub const RTSP_IDLE_TIME:Duration = Duration::from_secs(1);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FlirMsg{
    RequestNal,
    Nal(Vec<u8>),
}

#[async_trait]
pub trait IrSource{
    /// Will return a complete nal frame using the retina avc demuxer
    async fn next(&self) -> Vec<u8>;
    /// Will return a complete rgb image by polling the ir cam via
    /// next until a successful decode is achieved
    async fn image(&self) -> DynamicImage;
}

/// The driver for the Flir A50
pub struct A50<S:IrSource>{
    source: S,
}

/// Will attempt to establish a RTSP session with a flir camera
pub struct RtspSession{
    session: Mutex<Option<flume::Sender<flume::Sender<Vec<u8>>>>>,
    link: (flume::Sender<Vec<u8>>, flume::Receiver<Vec<u8>>)
}

/// Will conduct communication over the network to gather data needed for 
/// ir image reconstruction
pub struct A50Link{
}

impl<S:IrSource> A50<S>{
    pub fn new(source: S) -> Arc<Self> {
        Arc::new(Self{
            source,
        })
    }
}

impl RtspSession{
    pub async fn new(rt: Option<Arc<Runtime>>) -> Arc<Self> {
        let ip = match local_ip_address::local_ip().expect("Could not get local ip addr"){
            std::net::IpAddr::V4(i) => i,
            std::net::IpAddr::V6(i) => i.to_ipv4_mapped().expect("Could net get ipv4 addr"),
        };
        let subnet = Ipv4Addr::new(255,255,255,0);
        let scanner = Scanner::new_with_config(rt, ip.into(), subnet, (554,554), 256);
        let rtsp = Arc::new(
            Self{
                session: Mutex::new(None),
                link: flume::unbounded(),
            }
        );
        scanner.set_handler(Arc::new(rtsp.clone())).await;
        scanner.dispatch().await;
        
        rtsp
    }
    
}

#[async_trait]
impl IrSource for RtspSession{
    fn next(&self) ->  trait {
        todo!()
    }

    fn image<'life0,'async_trait>(&'life0 self) ->  core::pin::Pin<Box<dyn core::future::Future<Output = DynamicImage> + core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }
}

#[async_trait]
impl ScannerStreamHandler for Arc<RtspSession>{
    async fn handle(&self, stream: TcpStream){
        // We must attempt to establish an rtsp stream
        let peer_addr = stream.peer_addr().expect("Could not get peer addr for rtsp");
        let url = match Url::parse(&format!("rtsp://:@{}:554/avc", peer_addr.ip())){
            Ok(u) => u,
            Err(_) => return,
        };

        
        // We must first attempt to stream an image from the flir
        let mut options = SessionOptions::default();
        options = options.user_agent(String::from("demo"));

        let mut session = client::Session::describe(url, options)
            .await
            .expect("Could not establish session with A50");
        let options = SetupOptions::default();
        session
            .setup(0, options)
            .await
            .expect("Could not initiate stream with A50");
        let options = PlayOptions::default();
        let err = format!("Could not start playing string {}", 0);
        let play = session.play(options).await.expect(&err);
        let demux = play.demuxed().expect("Could not demux the playing stream");
        tokio::pin!(demux);
        
        println!("RTSP stream established on {}", peer_addr);
        
        let (tx, rx) = flume::unbounded();
        *self.session.lock().await = Some(tx);

        while Arc::strong_count(&self) > 1{
            let request;
            tokio::select!{
                _ = sleep(RTSP_IDLE_TIME) => {continue;}
                val = rx.recv_async() => {
                    match val{
                        Ok(r) => request = r,
                        Err(_) => {break;},
                    }
                }
            }
            
            let mut encoded_data = vec![];

            let frame = demux.next().await;
            match frame{
                Some(f) => {
                    if let Ok(retina::codec::CodecItem::VideoFrame(v)) = f{
                        encoded_data.extend_from_slice(v.data());
                    }
                },
                None => {},
            }

            let _  = request.send_async(encoded_data).await;
        }
    }
    
}

