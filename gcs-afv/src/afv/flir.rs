use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use async_trait::async_trait;
use eframe::{egui::{self, plot::{PlotImage, PlotPoint, Points, Arrows}}, epaint::{TextureHandle, Color32}};
use futures::StreamExt;
use glam::Vec2;
use image::{DynamicImage, ImageBuffer};
use openh264::{decoder::Decoder, nal_units, to_bitstream_with_001_be};
use retina::client::{self, PlayOptions, SessionOptions, SetupOptions};
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpStream,
    runtime::{Runtime, Handle},
    sync::RwLock,
    time::{sleep, Duration},
};
use url::Url;

use crate::{
    gui::GuiElement,
    network::{AfvMessage, ComEngine, ComEngineService},
    scanner::{Scanner, ScannerStreamHandler},
};

pub const RTSP_IDLE_TIME: Duration = Duration::from_secs(1);
pub const LINK_IDLE_TIME: Duration = Duration::from_secs(10);
pub const FLIR_ATTEMPT_CONNECT_TIME: Duration = Duration::from_secs(3);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FlirMsg {
    /// Opens a stream with a specified update frequency
    /// false = IR stream, true = Visual stream
    OpenStream(bool),
    #[serde(with = "serde_bytes")]
    Nal(Vec<u8>),
    CloseStream,
    SetFilter(u8),
    ReportFilter,
    AfvFilter(u8),
}

#[async_trait]
/// This trait embodies what it takes to drive the underlying device
pub trait Controller: Send + Sync{
    /// Will start a nal packet stream
    fn stream(self: Arc<Self>, visual: bool) -> flume::Receiver<Vec<u8>>;
}

/// Directly controls the a50
pub struct Actuator{
    peer_addr: RwLock<Option<SocketAddr>>,
    open_stream: RwLock<bool>,
    _com: Option<Arc<ComEngine<AfvMessage>>>,    
}

/// Sends commands and recieved data from an actuator through a comengine
pub struct Link{
    com: Arc<ComEngine<AfvMessage>>,    
    network_channel: RwLock<Option<flume::Sender<Vec<u8>>>>,
}

/// High level system for interacting with a flir
pub struct Flir{
    handle: Handle,
    com: Option<Arc<ComEngine<AfvMessage>>>,
    open: RwLock<bool>,
    live: RwLock<bool>,
    controller: Arc<dyn Controller>,
    image_data: RwLock<DynamicImage>,
    processed_image: RwLock<DynamicImage>,
    gui_image: RwLock<Option<TextureHandle>>,
    upper_centroid: RwLock<Vec2>,
    lower_centroid: RwLock<Vec2>,
    filter_value: RwLock<u8>,
    afv_filter: RwLock<u8>,
    show_filtered: RwLock<bool>,
    do_filtering: RwLock<bool>,
}

impl Flir{
    pub async fn actuated(com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Flir> {
        let controller = Actuator::new(com.clone()).await;
        let flir = Arc::new(Self{
            open: RwLock::new(false),
            controller,
            image_data: RwLock::new(DynamicImage::new_rgb8(650,480)),
            gui_image: RwLock::new(None),
            live: RwLock::new(false),
            handle: Handle::current(),
            upper_centroid: Default::default(),
            lower_centroid: Default::default(),
            processed_image: RwLock::new(DynamicImage::new_rgb8(100,100)),
            filter_value: Default::default(),
            show_filtered: Default::default(),
            com: com.clone(),
            afv_filter: Default::default(),
            do_filtering: Default::default(),
        });
        if let Some(com) = com{
            com.add_listener(flir.clone()).await;
        }
        flir
    }
    pub fn actuated_blocking(rt: Arc<Runtime>, com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Flir> {
        rt.block_on(Self::actuated(com))
    }
    pub async fn linked(com: Arc<ComEngine<AfvMessage>>) -> Arc<Flir> {
        let controller = Link::new(com.clone()).await;
        let flir = Arc::new(Self{
            open: RwLock::new(false),
            controller,
            image_data: RwLock::new(DynamicImage::new_rgb8(640,480)),
            gui_image: RwLock::new(None),
            live: RwLock::new(false),
            handle: Handle::current(),
            upper_centroid: Default::default(),
            lower_centroid: Default::default(),
            processed_image: RwLock::new(DynamicImage::new_rgb8(100,100)),
            filter_value: Default::default(),
            show_filtered: Default::default(),
            com: Some(com.clone()),
            afv_filter: Default::default(),
            do_filtering: Default::default(),
        });
        com.add_listener(flir.clone()).await;
        flir
    }
    fn analyze_image(self: Arc<Self>){
        let image = self.image_data.blocking_read().clone();
        let mut processed_image = self.processed_image.blocking_write();
        *processed_image = DynamicImage::new_rgb8(image.width(), image.height());
        let rgb = image.as_rgb8().expect("Could not represent as rgb image");
        let processed_rgb = processed_image.as_mut_rgb8().expect("Could not represent as rgb image");
        let mut pixels = Vec::with_capacity((rgb.width()*rgb.height()*3) as usize);

        let filter_value = *self.filter_value.blocking_read();

        for (x,y,pix) in rgb.enumerate_pixels().filter(|(_,_,pix)| {pix.0[0] > filter_value}){
            processed_rgb.put_pixel(x,y,*pix);
            pixels.push((x,y))
        }

        drop(image);
        drop(processed_image);

        if pixels.len() <= 10{
            *self.lower_centroid.blocking_write() = Vec2::new(0.0, 0.0);
            *self.upper_centroid.blocking_write() = Vec2::new(0.0, 0.0);
            return;
        }


        pixels.sort_unstable_by_key(|(_, y)| *y);
        let pix_count = pixels.len();

        let (lpix, upix) = pixels.split_at_mut(pix_count/2);
        
        let lower_y_median = lpix[lpix.len()/2];
        lpix.sort_unstable_by_key(|(x,_)| *x);
        let lower_x_median = lpix[lpix.len()/2];
        *self.lower_centroid.blocking_write() = Vec2::new(lower_x_median.0 as f32, lower_y_median.1 as f32);
        
        let upper_y_median = upix[upix.len()/2];
        upix.sort_unstable_by_key(|(x,_)| *x);
        let upper_x_median = upix[upix.len()/2];
        *self.upper_centroid.blocking_write() = Vec2::new(upper_x_median.0 as f32, upper_y_median.1 as f32);

    }
    
    pub fn linked_blocking(rt: Arc<Runtime>, com: Arc<ComEngine<AfvMessage>>) -> Arc<Flir> {
        rt.block_on(Self::linked(com))
    }
    pub async fn live_feed(self: Arc<Self>, visual: bool){
        *self.live.write().await = true;
        tokio::spawn(self.feed(visual));
    }
    pub async fn stop_feed(&self){
        *self.live.write().await = false;
    }
    pub fn stop_feed_blocking(&self){
        *self.live.blocking_write() = false;
    }
    pub async fn feed(self: Arc<Self>, visual: bool){
        let stream = self.controller.clone().stream(visual);
        let mut successful_image = false;
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
                    let mut rgb_data = vec![0; image_size.0*image_size.1*3];
                    yuv.write_rgb8(&mut rgb_data);
                    let image_data = match ImageBuffer::from_raw(
                        image_size.0 as u32,
                        image_size.1 as u32,
                        rgb_data,
                    ) {
                        Some(i) => i,
                        None => continue,
                    };
                    successful_image = true;
                    *self.image_data.write().await = DynamicImage::ImageRgb8(image_data.clone());
                    *self.gui_image.write().await = None;
                    // self.clone().analyze_image().await;
                    let flir = self.clone();
                    if *self.do_filtering.read().await{
                        self.handle.spawn_blocking(move || {flir.clone().analyze_image()});
                    }
                }
            }

            if !*self.live.read().await && successful_image{
                break;
            }
        }
    }

    pub fn get_gui_image(&self, ui: &mut egui::Ui) -> TextureHandle{
        let mut gui_image = self.gui_image.blocking_write();
        if let Some(i) = &(*gui_image){
            return i.clone();
        }

        let image;
        if *self.show_filtered.blocking_read(){
           image = self.processed_image.blocking_read();
        }
        else{
           image = self.image_data.blocking_read();
        }
        let rgb = image.as_rgb8().unwrap();
        let pixels = rgb.as_flat_samples();
        let size = [rgb.width() as usize, rgb.height() as usize];
        let color_image = egui::ColorImage::from_rgb(size, pixels.as_slice());
        let handle = ui.ctx().load_texture("Flir Output", color_image, Default::default());

        *gui_image = Some(handle.clone());
        handle
    }
}

#[async_trait]
impl ComEngineService<AfvMessage> for Flir{
    async fn notify(self: Arc<Self>, _com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::Flir(FlirMsg::SetFilter(val)) = msg{
            *self.filter_value.write().await = val;
            return;
        }
        
        if let AfvMessage::Flir(FlirMsg::ReportFilter) = msg{
            if let Some(com) = &self.com{
                let _ = com.send(AfvMessage::Flir(FlirMsg::AfvFilter(*self.filter_value.read().await))).await;
            }
            return;
        }
        
        if let AfvMessage::Flir(FlirMsg::AfvFilter(val)) = msg{
            *self.afv_filter.write().await = val;
            return;
        }
    }
}

impl GuiElement for Arc<Flir>{
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        let open = self.open.blocking_write();
        if !*open{
            *self.live.blocking_write() = false;
        }
        open
    }

    fn name(&self) -> String {
        "Flir".into()
    }

    fn render(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui|{
            let mut live = self.live.blocking_write();

            if *live{
                if ui.button("Stop feed").clicked(){
                    *live = false;
                }
            }
            else{
                if ui.button("Start IR Feed").clicked(){
                    self.handle.spawn(self.clone().live_feed(false));
                }
                if ui.button("Start Visual Feed").clicked(){
                    self.handle.spawn(self.clone().live_feed(true));
                }
                if ui.button("Update IR Image").clicked(){
                    self.handle.spawn(self.clone().feed(false));
                }
                if ui.button("Update Visual Image").clicked(){
                    self.handle.spawn(self.clone().feed(true));
                }
            }
        });
        ui.horizontal(|ui|{
            let mut filter = self.show_filtered.blocking_write();
            let mut filter_value = self.filter_value.blocking_write();
            let mut drag_value = *filter_value;
            let mut toggle_filtering = self.do_filtering.blocking_write();
            
            ui.toggle_value(&mut toggle_filtering, "Toggle filtering");
            ui.label("Filter Value: ");
            let drag = egui::widgets::DragValue::new(&mut drag_value).clamp_range(0..=u8::MAX);
            ui.add(drag);
            if let Some(com) = &self.com{
                if ui.button("Send filter value to afv").clicked(){
                    self.handle.spawn(com.clone().send_into(AfvMessage::Flir(FlirMsg::SetFilter(drag_value))));
                }
            }
            *filter_value = drag_value;
            ui.toggle_value(&mut filter, "Show filtered image");
            
        });
        if let Some(com) = &self.com{
            ui.horizontal(|ui|{
                let filter_value = self.afv_filter.blocking_read();
                ui.label(format!("Afv filter Value: {}", *filter_value));
                if ui.button("Request Afv filter value").clicked(){
                    self.handle.spawn(com.clone().send_into(AfvMessage::Flir(FlirMsg::ReportFilter)));
                }
            });
        }
        let gui_image = self.get_gui_image(ui);
        let gui_image_size = gui_image.size_vec2();
        // let size = ui.available_size();
        let lower_centroid = *self.lower_centroid.blocking_read();
        let upper_centroid = *self.upper_centroid.blocking_read();

        let lower_centroid = [lower_centroid.x as f64, -lower_centroid.y as f64];
        let upper_centroid = [upper_centroid.x as f64, -upper_centroid.y as f64];
        let points = Points::new(vec![lower_centroid, upper_centroid])
        .shape(egui::plot::MarkerShape::Circle)
        .filled(true)
        .radius(5.0)
        .name("Centroids");
        let arrow = Arrows::new(vec![upper_centroid], vec![lower_centroid])
        .color(Color32::RED)
        .name("Fire Axis");
        // ui.image(gui_image.id(), ui.available_size());
        egui::widgets::plot::Plot::new("Flir plot")
            .show_background(false)
            .data_aspect(1.0)
            .include_x(gui_image_size.x)
            .include_y(-gui_image_size.y)
            .label_formatter(|_name, value| {format!("    {:.0}, {:.0}", value.x, value.y.abs())})
            .y_axis_formatter(|y, _range| {y.abs().to_string()})
            .show(ui, |ui|{
            let image = PlotImage::new(gui_image.id(), PlotPoint::new(gui_image_size.x/2.0,-gui_image_size.y/2.0), gui_image_size);
            ui.image(image);
            ui.points(points);
            ui.arrows(arrow);
        });
    }
}

/// Actuator Impl

impl Actuator{
    pub async fn new(com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Actuator> {
        let controller = Arc::new(Self{
            _com: com.clone(),
            peer_addr: RwLock::new(None),
            open_stream: RwLock::new(false),
        });
        
        let ip = match local_ip_address::local_ip().expect("Could not get local ip addr") {
            std::net::IpAddr::V4(i) => i,
            std::net::IpAddr::V6(i) => i.to_ipv4_mapped().expect("Could net get ipv4 addr"),
        };
        println!("FLIR ACTUATOR: Looking for flir in network {}", ip);
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
            println!("Attempting Flir connection");
            let _ = scanner.request_dispatch().await;
            sleep(FLIR_ATTEMPT_CONNECT_TIME).await;
        }
        println!("FLIR ACTUATOR: Connected to FLIR at {:?}", *self.peer_addr.read().await);
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
    fn stream(self:Arc<Self>, visual: bool) -> flume::Receiver<Vec<u8> >  {
        let (tx, rx) = flume::unbounded();
        let a50 = self.clone();
        tokio::spawn(async move {
            let peer_addr = match *a50.peer_addr.read().await {
                Some(a) => a,
                None => {println!("No peer addr available for rtsp stream");return},
            };

            // We must attempt to establish an rtsp stream
            let url;
            if visual{
                url = match Url::parse(&format!("rtsp://:@{}:554/avc/ch1", peer_addr.ip())) {
                    Ok(u) => u,
                    Err(_) => return,
                };
            }
            else{
                url = match Url::parse(&format!("rtsp://:@{}:554/avc", peer_addr.ip())) {
                    Ok(u) => u,
                    Err(_) => return,
                };
            }

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
            println!("FLIR ACTUATOR: Rtsp stream opened on {}", peer_addr);

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
            println!("FLIR ACTUATOR: Rtsp stream closed");
        });
        rx
    }
}

#[async_trait]
impl ComEngineService<AfvMessage> for Actuator{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::Flir(FlirMsg::OpenStream(visual)) = msg{
            println!("FLIR ACTUATOR: Starting Flir network stream");
            {
                *self.open_stream.write().await = true;
            }
            let stream = self.clone().stream(visual);
            while *self.open_stream.read().await{
               if let Ok(p) = stream.recv_async().await{
                    // tokio::spawn(com.clone().send_parallel(AfvMessage::FlirMsg(FlirMsg::Nal(p))));
                    let _ = com.send(AfvMessage::Flir(FlirMsg::Nal(p))).await;
                    continue;
                } 
                *self.open_stream.write().await = false;
            }
            println!("FLIR ACTUATOR: Stopping Flir network stream");
            return;
        }
        if let AfvMessage::Flir(FlirMsg::CloseStream) = msg{
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
    fn stream(self:Arc<Self>, visual: bool) -> flume::Receiver<Vec<u8> >  {
        let (n_tx, n_rx) = flume::unbounded();
        let (s_tx, s_rx) = flume::unbounded();
        let link = self.clone();
        tokio::spawn(async move{
            println!("FLIR LINK: Attempting remote flir network stream");
            let _ = link.com.send(AfvMessage::Flir(FlirMsg::OpenStream(visual))).await;
            {
                *self.network_channel.write().await = Some(n_tx);
            }
            loop{
                let p;
                tokio::select!{
                    _ = sleep(LINK_IDLE_TIME) => {
                        println!("FLIR LINK: Remote flir network stream timeout");
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
            let _ = link.com.send(AfvMessage::Flir(FlirMsg::CloseStream)).await;
            *link.network_channel.write().await = None;
            println!("FLIR LINK: Closed remote flir network stream");
        });
        s_rx
    }
}

#[async_trait]
impl ComEngineService<AfvMessage> for Link{
    async fn notify(self: Arc<Self>, _com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::Flir(FlirMsg::Nal(p)) = msg{
            if let Some(tx) = &(*self.network_channel.read().await){
                let _ = tx.send_async(p).await;
            }
        }
    }
}
