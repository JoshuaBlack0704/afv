use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use async_trait::async_trait;
use eframe::{egui, epaint::TextureHandle};
use futures::StreamExt;
use image::{DynamicImage, ImageBuffer};
use openh264::{decoder::Decoder, nal_units, to_bitstream_with_001_be};
use retina::client::{self, PlayOptions, SessionOptions, SetupOptions};
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpStream,
    runtime::Runtime,
    sync::RwLock,
    time::{sleep, Duration},
};
use url::Url;

use crate::{
    gui::GuiElement,
    network::{AfvMessage, ComEngine, AfvComService},
    scanner::{Scanner, ScannerStreamHandler},
};

pub const RTSP_IDLE_TIME: Duration = Duration::from_secs(1);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FlirMsg {
    OpenStream,
    Nal(Vec<u8>),
    CloseStream,
}

#[async_trait]
pub trait IrSource: Send + Sync {
    /// Will return a complete nal frame using the retina avc demuxer
    fn stream(&self) -> flume::Receiver<Vec<u8>>;
    /// Will return a complete rgb image by polling the ir cam via
    /// next until a successful decode is achieved
    async fn image(&self) -> DynamicImage;
}

/// The driver for the Flir A50
pub struct A50<S: IrSource> {
    rt: Arc<Runtime>,
    open: RwLock<bool>,
    source: S,
    image_data: RwLock<DynamicImage>,
    gui_image: RwLock<Option<TextureHandle>>,
    network_stream: RwLock<bool>,
}

/// Will attempt to establish a RTSP session with a flir camera
pub struct RtspSession {
    peer_addr: RwLock<Option<SocketAddr>>,
}

/// Will conduct communication over the network to gather data needed for
/// ir image reconstruction
pub struct A50Link {
    com: Arc<ComEngine<AfvMessage>>,
    network_stream: RwLock<Option<flume::Sender<Vec<u8>>>>,
}





impl<S: IrSource + 'static> A50<S> {
    pub fn new(rt: Option<Arc<Runtime>>, source: S) -> Arc<Self> {
        let rt = match rt {
            Some(r) => r,
            None => Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Could not build runtime"),
            ),
        };
        Arc::new(Self {
            source,
            open: RwLock::new(false),
            image_data: RwLock::new(DynamicImage::default()),
            gui_image: RwLock::new(None),
            rt,
            network_stream: RwLock::new(false),
        })
    }
    pub fn load_gui_image(&self, ui: &egui::Ui) -> TextureHandle {
        let image = self.image_data.blocking_read().to_rgb8();
        let pixels = image.as_flat_samples();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = eframe::egui::ColorImage::from_rgb(size, pixels.as_slice());
        let texture = ui
            .ctx()
            .load_texture("Flir Output", color_image, Default::default());
        texture
    }
    pub fn refresh_interval(self: Arc<Self>, interval: Duration) {
        if interval == Duration::from_secs(0) {
            self.rt.spawn(self.clone().periodic_refresh(None));
        } else {
            self.rt.spawn(self.clone().periodic_refresh(Some(interval)));
        }
    }
    pub async fn periodic_refresh(self: Arc<Self>, interval: Option<Duration>) {
        while Arc::strong_count(&self) > 1 {
            let image = self.source.image().await;
            *self.image_data.write().await = image;
            *self.gui_image.write().await = None;
            match interval {
                Some(i) => {
                    sleep(i).await;
                }
                None => {}
            }
        }
    }
}

impl<S: IrSource + 'static> GuiElement for A50<S> {
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        self.open.blocking_write()
    }

    fn name(&self) -> String {
        "A50".into()
    }

    fn render(&self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        let mut gui_lock = self.gui_image.blocking_write();
        let gui_image = gui_lock.get_or_insert(self.load_gui_image(ui));
        ui.image(gui_image.id(), ui.available_size());
    }
}

#[async_trait]
impl<S:IrSource + 'static> AfvComService<AfvMessage> for A50<S>{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::FlirMsg(FlirMsg::OpenStream) = msg{
            {
                let mut open = self.network_stream.write().await;
                if *open{
                    return;
                }
                else{
                    *open = true;
                }
            }

            let stream = self.source.stream();
            tokio::spawn(async move {
                while let Ok(p) = stream.recv_async().await{
                    if !*self.network_stream.read().await{
                        return;
                    }

                    let packet = FlirMsg::Nal(p);
                    let _ = com.send(AfvMessage::FlirMsg(packet)).await;
                }
                *self.network_stream.write().await = false;
            });
            return;
        }
        
        if let AfvMessage::FlirMsg(FlirMsg::CloseStream) = msg{
            *self.network_stream.write().await = false;
        }
        
        if let AfvMessage::Closed = msg{
            *self.network_stream.write().await = false;
        }
    }
}

impl RtspSession {
    pub async fn new(rt: Option<Arc<Runtime>>) -> Arc<Self> {
        let ip = match local_ip_address::local_ip().expect("Could not get local ip addr") {
            std::net::IpAddr::V4(i) => i,
            std::net::IpAddr::V6(i) => i.to_ipv4_mapped().expect("Could net get ipv4 addr"),
        };
        println!("Looking for rtsp stream on network {}", ip);
        let subnet = Ipv4Addr::new(255, 255, 255, 0);
        let scanner = Scanner::new_with_config(rt, ip.into(), subnet, (554, 554), 256).await;
        let rtsp = Arc::new(Self {
            peer_addr: RwLock::new(None),
        });

        tokio::spawn(rtsp.clone().attempt_connection(scanner));
        rtsp
    }
    pub fn new_blocking(rt: Arc<Runtime>) -> Arc<RtspSession> {
        rt.block_on(Self::new(Some(rt.clone())))
    }
    pub async fn attempt_connection(self: Arc<Self>, scanner: Arc<Scanner>) {
        scanner.set_handler(Arc::new(self.clone())).await;
        let mut connected = false;
        while !connected {
            match *self.peer_addr.read().await {
                Some(_) => connected = true,
                None => {
                    println!("Attempting connection to flir camera");
                    tokio::spawn(scanner.clone().dispatch());
                    sleep(RTSP_IDLE_TIME).await;
                }
            }
        }
        println!("Flir connection task stopping");
    }
}

#[async_trait]
impl IrSource for Arc<RtspSession> {
    fn stream(&self) -> flume::Receiver<Vec<u8>> {
        let (tx, rx) = flume::unbounded();
        let a50 = self.clone();
        tokio::spawn(async move {
            let peer_addr = match *a50.peer_addr.read().await {
                Some(a) => a,
                None => return,
            };

            println!("Recieved rtsp stream request");
            // We must attempt to establish an rtsp stream
            let url = match Url::parse(&format!("rtsp://:@{}:554/avc", peer_addr.ip())) {
                Ok(u) => u,
                Err(_) => return,
            };

            // We must first attempt to stream an image from the flir
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
            println!("Rtsp stream opened on {}", peer_addr);

            tokio::pin!(demux);

            while !tx.is_disconnected() {
                let mut encoded_data = vec![];

                let frame = demux.next().await;
                match frame {
                    Some(f) => {
                        if let Ok(retina::codec::CodecItem::VideoFrame(v)) = f {
                            encoded_data.extend_from_slice(v.data());
                            if let Err(_) = tx.send_async(encoded_data).await {
                                break;
                            }
                        }
                    }
                    None => {}
                };
            }
            println!("Rtsp stream request closed");
        });
        rx
    }
    async fn image(&self) -> DynamicImage {
        println!("Requesting new image from rtsp stream");
        let mut image: Option<DynamicImage> = None;
        let mut decoder = match Decoder::new() {
            Ok(d) => d,
            Err(_) => {
                println!("Could not create decoder");
                return DynamicImage::default();
            }
        };
        let stream = self.stream();
        while let None = image {
            let packet = match stream.recv_async().await {
                Ok(p) => p,
                Err(_) => {
                    println!("Rtsp stream early disconnect");
                    return DynamicImage::default();
                }
            };

            let mut nal = Vec::with_capacity(packet.len());
            println!("Recieved nal packet");

            to_bitstream_with_001_be::<u32>(&packet, &mut nal);

            for nal in nal_units(&nal) {
                if let Ok(Some(yuv)) = decoder.decode(nal) {
                    println!("Successfully decoded image");
                    let image_size = yuv.dimension_rgb();
                    let mut rgb_data = vec![0; image_size.0 * image_size.1 * 3];
                    yuv.write_rgb8(&mut rgb_data);
                    let image_data = match ImageBuffer::from_raw(
                        image_size.0 as u32,
                        image_size.1 as u32,
                        rgb_data,
                    ) {
                        Some(i) => i,
                        None => return DynamicImage::default(),
                    };
                    image = Some(DynamicImage::ImageRgb8(image_data));
                }
            }
        }

        if let Some(i) = image {
            return i;
        } else {
            return DynamicImage::default();
        }
    }
}

#[async_trait]
impl ScannerStreamHandler for Arc<RtspSession> {
    async fn handle(&self, stream: TcpStream) {
        // We must attempt to establish an rtsp stream
        let peer_addr = match stream.peer_addr() {
            Ok(a) => a,
            Err(_) => return,
        };
        let a50 = self.clone();
        tokio::spawn(async move {
            *a50.peer_addr.write().await = Some(peer_addr);
        });
    }
}

impl A50Link{
    pub fn new(com: Arc<ComEngine<AfvMessage>>){
        let link = Arc::new(Self{
            com: com.clone(),
            network_stream: RwLock::new(None),
        });
        
        com.add_listener_blocking(link.clone());
    }
}

#[async_trait]
impl AfvComService<AfvMessage> for A50Link{
    async fn notify(self: Arc<Self>, _com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::FlirMsg(FlirMsg::Nal(p)) = msg{
            if let Some(s) = &(*self.network_stream.read().await){
                let _ = s.send_async(p).await;
            }
            return;
        }

        if let AfvMessage::Closed = msg{
            *self.network_stream.write().await = None;
        }
    }
}

#[async_trait]
impl IrSource for A50Link{
    
    fn stream(&self) -> flume::Receiver<Vec<u8>> {
        let (s_tx, s_rx) = flume::unbounded();
        let (n_tx, n_rx) = flume::unbounded();
        tokio::spawn(async move {
        });
        s_rx
    }
    
    async fn image(&self) -> DynamicImage {
        println!("Requesting new image from rtsp stream");
        let mut image: Option<DynamicImage> = None;
        let mut decoder = match Decoder::new() {
            Ok(d) => d,
            Err(_) => {
                println!("Could not create decoder");
                return DynamicImage::default();
            }
        };
        let stream = self.stream();
        while let None = image {
            let packet = match stream.recv_async().await {
                Ok(p) => p,
                Err(_) => {
                    println!("Rtsp stream early disconnect");
                    return DynamicImage::default();
                }
            };

            let mut nal = Vec::with_capacity(packet.len());
            println!("Recieved nal packet");

            to_bitstream_with_001_be::<u32>(&packet, &mut nal);

            for nal in nal_units(&nal) {
                if let Ok(Some(yuv)) = decoder.decode(nal) {
                    println!("Successfully decoded image");
                    let image_size = yuv.dimension_rgb();
                    let mut rgb_data = vec![0; image_size.0 * image_size.1 * 3];
                    yuv.write_rgb8(&mut rgb_data);
                    let image_data = match ImageBuffer::from_raw(
                        image_size.0 as u32,
                        image_size.1 as u32,
                        rgb_data,
                    ) {
                        Some(i) => i,
                        None => return DynamicImage::default(),
                    };
                    image = Some(DynamicImage::ImageRgb8(image_data));
                }
            }
        }

        if let Some(i) = image {
            return i;
        } else {
            return DynamicImage::default();
        }
    }
}
