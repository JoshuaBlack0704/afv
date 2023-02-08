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
pub const LINK_IDLE_TIME: Duration = Duration::from_secs(10);

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

#[async_trait]
/// This trait embodies what it takes to drive the underlying device
pub trait Controller: Send + Sync{
    /// Will start a nal packet stream
    fn stream(self: Arc<Self>) -> flume::Receiver<Vec<u8>>;
}

/// Directly controls the a50
pub struct Actuator{
    peer_addr: RwLock<Option<SocketAddr>>,
    open_stream: RwLock<bool>,
    com: Option<Arc<ComEngine<AfvMessage>>>,    
}

/// Sends commands and recieved data from an actuator through a comengine
pub struct Link{
    com: Arc<ComEngine<AfvMessage>>,    
    network_channel: RwLock<Option<flume::Sender<Vec<u8>>>>,
}

/// High level system for interacting with a flir
pub struct Flir{
    open: RwLock<bool>,
    live: RwLock<bool>,
    controller: Arc<dyn Controller>,
    image_data: RwLock<DynamicImage>,
    gui_image: RwLock<Option<TextureHandle>>,
}

impl Flir{
    pub async fn actuated(com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Flir> {
        let controller = Actuator::new(com).await;
        Arc::new(Self{
            open: RwLock::new(false),
            controller,
            image_data: RwLock::new(DynamicImage::new_rgb8(100,100)),
            gui_image: RwLock::new(None),
            live: RwLock::new(false),
        })
    }
    pub fn actuated_blocking(rt: Arc<Runtime>, com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Flir> {
        rt.block_on(Self::actuated(com))
    }
    pub async fn linked(com: Arc<ComEngine<AfvMessage>>) -> Arc<Flir> {
        let controller = Link::new(com).await;
        Arc::new(Self{
            open: RwLock::new(false),
            controller,
            image_data: RwLock::new(DynamicImage::new_rgb8(100,100)),
            gui_image: RwLock::new(None),
            live: RwLock::new(false),
        })
        
    }
    pub fn linked_blocking(rt: Arc<Runtime>, com: Arc<ComEngine<AfvMessage>>) -> Arc<Flir> {
        rt.block_on(Self::linked(com))
    }
    pub async fn live_feed(self: Arc<Self>){
        *self.live.write().await = true;
        tokio::spawn(self.feed());
    }
    pub async fn stop_feed(&self){
        *self.live.write().await = false;
    }
    pub async fn feed(self: Arc<Self>){
        let stream = self.controller.clone().stream();
        let mut decoder = match Decoder::new(){
            Ok(d) => d,
            Err(_) => return,
        };

        loop {
            let p = match stream.recv_async().await{
                Ok(p) => p,
                Err(_) => break,
            };

            let mut nal = Vec::with_capacity(p.len());
            to_bitstream_with_001_be::<u32>(&p, &mut nal);
            
            for p in nal_units(&nal){
                if let Ok(Some(yuv)) = decoder.decode(&p){
                    let image_size = yuv.dimension_rgb();
                    let mut rgb_data = vec![0; image_size.0*image_size.1];
                    yuv.write_rgb8(&mut rgb_data);
                    let image_data = match ImageBuffer::from_raw(
                        image_size.0 as u32,
                        image_size.1 as u32,
                        rgb_data,
                    ) {
                        Some(i) => i,
                        None => continue,
                    };
                    *self.image_data.write().await = DynamicImage::ImageRgb8(image_data);
                    *self.gui_image.write().await = None;
                }
            }

            if !*self.live.read().await{break}
        }
    }

    pub fn get_gui_image(&self, ui: &mut egui::Ui) -> TextureHandle{
        let mut gui_image = self.gui_image.blocking_write();
        if let Some(i) = &(*gui_image){
            return i.clone();
        }

        let image = self.image_data.blocking_read();
        let rgb = image.as_rgb8().unwrap();
        let pixels = rgb.as_flat_samples();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = egui::ColorImage::from_rgb(size, pixels.as_slice());
        let handle = ui.ctx().load_texture("Flir Output", color_image, Default::default());

        *gui_image = Some(handle.clone());
        handle
    }
}

impl GuiElement for Flir{
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        let open = self.open.blocking_write();
        *self.live.blocking_write() = *open;
        open
    }

    fn name(&self) -> String {
        "Flir".into()
    }

    fn render(&self, ui: &mut egui::Ui) {
        let gui_image = self.get_gui_image(ui);
        ui.image(gui_image.id(), ui.available_size());
    }
}


/// Actuator Impl

impl Actuator{
    pub async fn new(com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Actuator> {
        let controller = Arc::new(Self{
            com: com.clone(),
            peer_addr: RwLock::new(None),
            open_stream: RwLock::new(false),
        });
        
        let ip = match local_ip_address::local_ip().expect("Could not get local ip addr") {
            std::net::IpAddr::V4(i) => i,
            std::net::IpAddr::V6(i) => i.to_ipv4_mapped().expect("Could net get ipv4 addr"),
        };
        println!("Looking for flir in network {}", ip);
        let subnet = Ipv4Addr::new(255, 255, 255, 0);
        let scanner = Scanner::new_with_config(ip.into(), subnet, (554, 554), 256).await;
        tokio::spawn(controller.clone().flir_repeat_connect(scanner));
        if let Some(com) = com{
            com.add_listener(controller.clone()).await;
        }
        controller
    }
    async fn flir_repeat_connect(self: Arc<Self>, scanner: Arc<Scanner>){
        scanner.set_handler(self.clone()).await;
        while let None = *self.peer_addr.read().await{
            let _ = scanner.request_dispatch().await;
            println!("Attempting Flir connection");
        }
        println!("Connected to FLIR at {:?}", *self.peer_addr.read().await);
    }
}

#[async_trait]
impl ScannerStreamHandler for Actuator{
    async fn handle(self: Arc<Self>, stream: TcpStream){
        let flir = self.clone();
        
        tokio::spawn(async move{
            match stream.peer_addr(){
                Ok(p) => {
                    *flir.peer_addr.write().await = Some(p);
                },
                Err(_) => {},
            }
        });
    }
}

impl Controller for Actuator{
    fn stream(self:Arc<Self>) -> flume::Receiver<Vec<u8> >  {
        let (tx, rx) = flume::unbounded();
        let a50 = self.clone();
        tokio::spawn(async move {
            let peer_addr = match *a50.peer_addr.read().await {
                Some(a) => a,
                None => {println!("No peer addr available for rtsp stream");return},
            };

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
            println!("Rtsp stream closed");
        });
        rx
    }
}

#[async_trait]
impl AfvComService<AfvMessage> for Actuator{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::FlirMsg(FlirMsg::OpenStream) = msg{
            println!("Starting Flir network stream");
            {
                *self.open_stream.write().await = true;
            }
            let stream = self.clone().stream();
            while *self.open_stream.read().await{
               if let Ok(p) = stream.recv_async().await{
                    let _ = com.send(AfvMessage::FlirMsg(FlirMsg::Nal(p))).await;
                    continue;
                } 
                *self.open_stream.write().await = false;
            }
            println!("Stopping Flir network stream");
            return;
        }
        if let AfvMessage::FlirMsg(FlirMsg::CloseStream) = msg{
            *self.open_stream.write().await = false;
            return;
        }
        
    }
}


/// Link Impl

impl Link{
    pub async fn new(com: Arc<ComEngine<AfvMessage>>) -> Arc<Link> {
        let controller = Arc::new(Self{
            com: com.clone(),
            network_channel: RwLock::new(None),
        });
        com.add_listener(controller.clone()).await;
        controller
    }
}

impl Controller for Link{
    fn stream(self:Arc<Self>) -> flume::Receiver<Vec<u8> >  {
        let (n_tx, n_rx) = flume::unbounded();
        let (s_tx, s_rx) = flume::unbounded();
        let link = self.clone();
        println!("Attempting remote flir network stream");
        tokio::spawn(async move{
            {
                *self.network_channel.write().await = Some(n_tx);
            }
            loop{
                let p;
                tokio::select!{
                    _ = sleep(LINK_IDLE_TIME) => {
                        println!("Remote flir network stream timeout");
                        break;
                    }
                    val = n_rx.recv_async() => {
                        p = val;
                    }
                }

                match p{
                    Ok(p) => {
                       if let Err(_) = s_tx.send_async(p).await{
                            break;
                        } 
                    },
                    Err(_) => break,
                }
            }
            let _ = link.com.send(AfvMessage::FlirMsg(FlirMsg::CloseStream)).await;
            *link.network_channel.write().await = None;
            println!("Closed remote flir network stream");
        });
        s_rx
    }
}

#[async_trait]
impl AfvComService<AfvMessage> for Link{
    async fn notify(self: Arc<Self>, _com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::FlirMsg(FlirMsg::Nal(p)) = msg{
            if let Some(tx) = &(*self.network_channel.read().await){
                let _ = tx.send_async(p).await;
            }
        }
    }
}


/// The driver for the Flir A50
pub struct A50 {
    rt: Arc<Runtime>,
    open: RwLock<bool>,
    source: Arc<dyn IrSource>,
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





impl A50{
    pub fn new(rt: Option<Arc<Runtime>>, source: Arc<dyn IrSource>) -> Arc<Self> {
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
        println!("Refresh");
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

impl GuiElement for A50{
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        self.open.blocking_write()
    }

    fn name(&self) -> String {
        "A50".into()
    }

    fn render(&self, ui: &mut egui::Ui) {
        let mut gui_lock = self.gui_image.blocking_write();
        let gui_image = gui_lock.get_or_insert(self.load_gui_image(ui));
        ui.image(gui_image.id(), ui.available_size());
    }
}

#[async_trait]
impl AfvComService<AfvMessage> for A50{
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
            println!("Attempting network rtsp stream");
                while let Ok(p) = stream.recv_async().await{
                    if !*self.network_stream.read().await{
                        break;
                    }

                    let packet = FlirMsg::Nal(p);
                    let _ = com.send(AfvMessage::FlirMsg(packet)).await;
                }
                *self.network_stream.write().await = false;
            println!("Finished network rtsp stream");
            });
            return;
        }
        
        if let AfvMessage::FlirMsg(FlirMsg::CloseStream) = msg{
            println!("Closing rtsp network stream");
            *self.network_stream.write().await = false;
        }
        
        if let AfvMessage::Closed = msg{
            *self.network_stream.write().await = false;
        }
    }
}

impl RtspSession {
    pub async fn new() -> Arc<Self> {
        let ip = match local_ip_address::local_ip().expect("Could not get local ip addr") {
            std::net::IpAddr::V4(i) => i,
            std::net::IpAddr::V6(i) => i.to_ipv4_mapped().expect("Could net get ipv4 addr"),
        };
        println!("Looking for rtsp stream on network {}", ip);
        let subnet = Ipv4Addr::new(255, 255, 255, 0);
        let scanner = Scanner::new_with_config(ip.into(), subnet, (554, 554), 256).await;
        let rtsp = Arc::new(Self {
            peer_addr: RwLock::new(None),
        });

        tokio::spawn(rtsp.clone().attempt_connection(scanner));
        rtsp
    }
    pub fn new_blocking(rt: Arc<Runtime>) -> Arc<RtspSession> {
        rt.block_on(Self::new())
    }
    pub async fn attempt_connection(self: Arc<Self>, scanner: Arc<Scanner>) {
        scanner.set_handler(self.clone()).await;
        let mut connected = false;
        while !connected {
            match *self.peer_addr.read().await {
                Some(_) => connected = true,
                None => {
                    println!("Attempting connection to flir camera");
                    let _ = scanner.request_dispatch().await;
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
                None => {println!("No peer addr available for rtsp stream");return},
            };

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
impl ScannerStreamHandler for RtspSession {
    async fn handle(self: Arc<Self>, stream: TcpStream) {
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
    pub async fn new(com: Arc<ComEngine<AfvMessage>>) -> Arc<A50Link> {
        let link = Arc::new(Self{
            com: com.clone(),
            network_stream: RwLock::new(None),
        });
        
        com.add_listener(link.clone()).await;
        link
    }
    pub fn new_blocking(com: Arc<ComEngine<AfvMessage>>) -> Arc<A50Link> {
        let link = Arc::new(Self{
            com: com.clone(),
            network_stream: RwLock::new(None),
        });
        
        com.add_listener_blocking(link.clone());
        link
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
impl IrSource for Arc<A50Link>{
    
    fn stream(&self) -> flume::Receiver<Vec<u8>> {
        let (s_tx, s_rx) = flume::unbounded();
        let (n_tx, n_rx) = flume::unbounded();
        let link = self.clone();
        tokio::spawn(async move {
            {
                *link.network_stream.write().await = Some(n_tx);
            }
            let _ = link.com.send(AfvMessage::FlirMsg(FlirMsg::OpenStream)).await;
            loop {
                let p;
                tokio::select!{
                    _ = sleep(LINK_IDLE_TIME) => {
                        println!("A50 link stream timeout");
                        break;
                    }
                    val = n_rx.recv_async() => {
                        p = val;
                    }
                }

                match p{
                    Ok(p) => {
                       if let Err(_) = s_tx.send_async(p).await{
                            break;
                        } 
                    },
                    Err(_) => break,
                }
                
            }
            println!("A50 link close");
            let _ = link.com.send(AfvMessage::FlirMsg(FlirMsg::CloseStream)).await;
            *link.network_stream.write().await = None;
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
            // println!("Recieved nal packet");

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
