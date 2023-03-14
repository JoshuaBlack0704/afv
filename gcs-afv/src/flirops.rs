use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use eframe::{epaint::{TextureHandle, Color32}, egui::{Ui, TopBottomPanel, self, CentralPanel, plot::{Points, Arrows, PlotImage, PlotPoint}}};
use glam::Vec2;
use image::{DynamicImage, ImageBuffer};
use openh264::{decoder::Decoder, to_bitstream_with_001_be, nal_units};
use rand::{thread_rng, Rng};
use tokio::{runtime::Handle, time::sleep, sync::RwLock};

use crate::{bus::{BusUuid, Bus, BusElement}, afvbus::AfvUuid, messages::{AfvCtlMessage, NetworkMessages, LocalMessages}, networkbus::{Network, Local}};

const FLIRFOV: (f32, f32) = (29.0,22.0);
pub struct FlirController<T>{
    bus_uuid: BusUuid,
    afv_uuid: RwLock<AfvUuid>,
    bus: Bus<AfvCtlMessage>,
    handle: Handle,
    _net: PhantomData<T>,
    
    flir_decoder: RwLock<Option<Decoder>>,
    flir_image: RwLock<DynamicImage>,
    flir_filtered_image: RwLock<Option<DynamicImage>>,
    flir_gui_image: RwLock<Option<TextureHandle>>,
    flir_filter: RwLock<bool>,
    flir_filter_level: RwLock<u8>,
    flir_analysis_barrier: RwLock<bool>,
    flir_centroids: RwLock<(Vec2, Vec2)>,
    flir_target_iterations: RwLock<u32>,
}


impl FlirController<Network>{
    pub async fn new(bus: Bus<AfvCtlMessage>) -> Arc<Self> {
        let flir = Arc::new(Self{
            bus_uuid: thread_rng().gen(),
            afv_uuid: Default::default(),
            bus: bus.clone(),
            handle: Handle::current(),
            _net: PhantomData,
            flir_decoder: Default::default(),
            flir_image: RwLock::new(DynamicImage::new_rgb8(300,300)),
            flir_gui_image: Default::default(),
            flir_filtered_image: Default::default(),
            flir_filter: Default::default(),
            flir_filter_level: Default::default(),
            flir_analysis_barrier: Default::default(),
            flir_centroids: Default::default(),
            flir_target_iterations: RwLock::new(2),
        });

        bus.add_element(flir.clone()).await;

        tokio::spawn(flir.clone().flir_stream_manager());

        flir
    }
    async fn flir_stream_manager(self: Arc<Self>){
        loop{
            if let Some(_) = & (*self.flir_decoder.read().await){
                self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Network(NetworkMessages::FlirStream(*self.afv_uuid.read().await))).await;
            }
            sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

impl FlirController<Local>{
    pub async fn new(bus: Bus<AfvCtlMessage>) -> Arc<Self> {
        let flir = Arc::new(Self{
            bus_uuid: thread_rng().gen(),
            afv_uuid: Default::default(),
            bus: bus.clone(),
            handle: Handle::current(),
            _net: PhantomData,
            flir_decoder: Default::default(),
            flir_image: RwLock::new(DynamicImage::new_rgb8(300,300)),
            flir_gui_image: Default::default(),
            flir_filtered_image: Default::default(),
            flir_filter: Default::default(),
            flir_filter_level: Default::default(),
            flir_analysis_barrier: Default::default(),
            flir_centroids: Default::default(),
            flir_target_iterations: RwLock::new(2),
        });

        bus.add_element(flir.clone()).await;

        tokio::spawn(flir.clone().flir_stream_manager());

        flir
    }
    async fn flir_stream_manager(self: Arc<Self>){
        loop{
            if let Some(_) = & (*self.flir_decoder.read().await){
                self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Local(LocalMessages::FlirStream(*self.afv_uuid.read().await))).await;
            }
            sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}


impl<T: Send + Sync + 'static> FlirController<T>{
    pub async fn enable_stream(&self){
        *self.flir_decoder.write().await = match Decoder::new(){
            Ok(d) => Some(d),
            Err(_) => return,
        }
    }
    pub async fn disable_stream(&self){
        *self.flir_decoder.write().await = None;
    }
    /// Returns the angle delta in degrees needed to center on the target centroid
    pub async fn get_target_offset(&self) -> (f32, f32){
        let target_centroid = self.flir_centroids.read().await.1;
        let image_width = self.flir_image.read().await.width();
        let image_height = self.flir_image.read().await.height();
        let center = Vec2::new((image_width as f32) / 2.0, (image_height as f32) / 2.0);
        let delta_pix = ( target_centroid - center) * Vec2::new(1.0, -1.0);
        let deg_pix_x = FLIRFOV.0 / image_width as f32;
        let deg_pix_y = FLIRFOV.1 / image_height as f32;

        let delta_x = delta_pix.x * deg_pix_x;
        let delta_y = delta_pix.y * deg_pix_y;

        (delta_x, delta_y)
    }
    pub fn get_target_offset_blocking(&self) -> (f32, f32) {
        self.handle.block_on(self.get_target_offset())
    }
    fn get_gui_image(&self, ui: &mut Ui) -> TextureHandle{
        let mut gui_image = self.flir_gui_image.blocking_write();
        if let Some(i) = &(*gui_image){
            return i.clone();
        }

        let image;
        // Select processed or direct image
        match & (*self.flir_filtered_image.blocking_read()){
            Some(i) => {
               image = i.clone().into_rgb8(); 
            },
            None => {
                image = self.flir_image.blocking_read().clone().into_rgb8();
            },
        }
        let pixels = image.as_flat_samples();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = egui::ColorImage::from_rgb(size, pixels.as_slice());
        let handle = ui.ctx().load_texture("Flir Output", color_image, Default::default());

        *gui_image = Some(handle.clone());
        handle
    }
    pub fn render_flir_display(&self, ui: &mut Ui){
        TopBottomPanel::top("Flir controls").show_inside(ui, |ui|{
            ui.horizontal(|ui|{
                // Stream controls
                let mut decoder = self.flir_decoder.blocking_write();
                match &mut (*decoder){
                    Some(_) => {
                        if ui.button("Stop Flir Stream").clicked(){
                            *decoder = None;
                        }
                    },
                    None => {
                        if ui.button("Start Flir Stream").clicked(){
                            if let Ok(d) = Decoder::new(){
                                *decoder = Some(d);
                            }
                        }
                    },
                }

                // Fliter controls
                let mut filtered_image = self.flir_filtered_image.blocking_write();
                match &(*filtered_image){
                    Some(_) => {
                        if ui.button("Hide filtered image").clicked(){
                            *filtered_image = None;
                        }
                    },
                    None => {
                        if ui.button("Show filtered image").clicked(){
                            *filtered_image = Some(DynamicImage::new_rgb8(100,100));
                        }
                    },
                }
                let mut filter_toggle = self.flir_filter.blocking_write();
                if !(*filter_toggle){
                    if ui.button("Enable filtering").clicked(){
                        *filter_toggle = true;
                    }
                }
                else {
                    if ui.button("Disable filtering").clicked(){
                        *filter_toggle = false;
                    }
                }
                let mut filter_level = self.flir_filter_level.blocking_write();
                let drag = egui::widgets::DragValue::new(&mut (*filter_level))
                    .clamp_range(0..=u8::MAX)
                    .prefix("Filter Level: ")
                    .speed(1.0);
                ui.add(drag);
                let mut filter_target_iterations = self.flir_target_iterations.blocking_write();
                let drag = egui::widgets::DragValue::new(&mut (*filter_target_iterations))
                    .clamp_range(1..=5)
                    .prefix("Filter Target Iterations: ")
                    .speed(0.1);
                ui.add(drag);
                if ui.button("Send filter level").clicked(){
                    self.handle.spawn(self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Network(NetworkMessages::FlirFilterLevel(*self.afv_uuid.blocking_read(), *filter_level))));
                    self.handle.spawn(self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Network(NetworkMessages::FlirTargetIterations(*self.afv_uuid.blocking_read(), *filter_target_iterations))));
                }
            });
        });
        CentralPanel::default().show_inside(ui, |ui|{
            self.plot_image(ui);
        });
    }
    pub fn plot_image(&self, ui: &mut Ui){
        let texture = self.get_gui_image(ui);
        let gui_image_size = texture.size_vec2();
        let (lower_centroid, upper_centroid) = *self.flir_centroids.blocking_read();
        
        let lower_centroid = [lower_centroid.x as f64, -lower_centroid.y as f64];
        let upper_centroid = [upper_centroid.x as f64, -upper_centroid.y as f64];
        let points = Points::new(vec![lower_centroid, upper_centroid])
        .shape(egui::plot::MarkerShape::Circle)
        .filled(true)
        .radius(5.0)
        .name("Centroids");
        let centroidal_arrow = Arrows::new(vec![upper_centroid], vec![lower_centroid])
        .color(Color32::RED)
        .name("Fire Axis");

        let delta_angles = self.get_target_offset_blocking();
        println!("{}", delta_angles.1);
        let center = [gui_image_size.x as f64 / 2.0, -gui_image_size.y as f64 / 2.0];
        let rot_x = [(gui_image_size.x + 5.0 * delta_angles.0) as f64 / 2.0, -gui_image_size.y as f64 / 2.0];
        let rot_y = [gui_image_size.x as f64 / 2.0, (-gui_image_size.y + 5.0 * delta_angles.1) as f64 / 2.0];
        let rotation_arrow = Arrows::new(vec![center, center], vec![rot_x, rot_y])
        .color(Color32::GOLD)
        .name("Rotation Arrow");
        
        egui::widgets::plot::Plot::new("Flir plot")
            .show_background(false)
            .data_aspect(1.0)
            .include_x(gui_image_size.x)
            .include_y(-gui_image_size.y)
            .label_formatter(|_name, value| {format!("    {:.0}, {:.0}", value.x, value.y.abs())})
            .y_axis_formatter(|y, _range| {y.abs().to_string()})
            .show(ui, |ui|{
            let image = PlotImage::new(texture.id(), PlotPoint::new(gui_image_size.x/2.0,-gui_image_size.y/2.0), gui_image_size);
            ui.image(image);
            ui.points(points);
            ui.arrows(centroidal_arrow);
            ui.arrows(rotation_arrow);
        });
        
    }
    async fn process_nal_packet(self: Arc<Self>, packet: Vec<u8>){
        if let Some(d) = &mut (*self.flir_decoder.write().await){
            let mut nal = Vec::with_capacity(packet.len());
            to_bitstream_with_001_be::<u32>(&packet, &mut nal);
            for packet in nal_units(&nal){
                if let Ok(Some(yuv)) = d.decode(&packet){
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
                    {
                        *self.flir_image.write().await = DynamicImage::ImageRgb8(image_data.clone());
                        *self.flir_gui_image.write().await = None;
                    }
                    // Since each process nal tasks is spawned in its own task, we can
                    // put cpu intensive tasks
                    if *self.flir_filter.read().await{
                        self.clone().analyze_image();
                    }
                }
            }
        }
    }
    fn analyze_image(self: Arc<Self>){
        self.clone().handle.spawn_blocking(move ||{
            {
                let mut barrier = self.flir_analysis_barrier.blocking_write();
                if *barrier{
                    return;
                }
                *barrier = true;
            }

            // Pull in our image
            let image = self.flir_image.blocking_read().clone().into_rgb8();
            let mut filtered_image = DynamicImage::new_rgb8(image.width(), image.height()).into_rgb8();
            let filter_level = *self.flir_filter_level.blocking_read();
            let mut pixels = Vec::with_capacity((image.width() * image.height() * 3) as usize);

            for (x,y,pix) in image.enumerate_pixels().filter(|(_,_,pix)| {pix.0[0] > filter_level}){
                filtered_image.put_pixel(x,y,*pix);
                pixels.push((x,y))
            }

            if let Some(i) = &mut (*self.flir_filtered_image.blocking_write()){
                *i = DynamicImage::from(filtered_image);
            }
        
            if pixels.len() <= 10{
                *self.flir_centroids.blocking_write() = Default::default();
                *self.flir_analysis_barrier.blocking_write() = false;
                return;
            }


            pixels.sort_unstable_by_key(|(_, y)| *y);
            let pix_count = pixels.len();

            let (lpix, upix) = pixels.split_at_mut(pix_count/2);
        
            let lower_y_median = lpix[lpix.len()/2];
            lpix.sort_unstable_by_key(|(x,_)| *x);
            let lower_x_median = lpix[lpix.len()/2];
            let lower_centroid = Vec2::new(lower_x_median.0 as f32, lower_y_median.1 as f32);


            let mut upix = upix.to_vec();
            let mut upper_centroid = Default::default();
            for _ in 0..*self.flir_target_iterations.blocking_read(){
                let upper_y_median = match upix.get(upix.len()/2){
                    Some(p) => *p,
                    None => (0,0),
                };
                upix.sort_unstable_by_key(|(x,_)| *x);
                let upper_x_median = match upix.get(upix.len()/2){
                    Some(p) => *p,
                    None => (0,0),
                };
                upper_centroid = Vec2::new(upper_x_median.0 as f32, upper_y_median.1 as f32);
                
                upix.sort_unstable_by_key(|(_,y)| *y);
                let (_,upper) = upix.split_at(upix.len()/2);
                upix = upper.to_vec();
            }
            
            *self.flir_centroids.blocking_write() = (lower_centroid, upper_centroid);
            *self.flir_analysis_barrier.blocking_write() = false;
        });
    }
}

#[async_trait]
impl BusElement<AfvCtlMessage> for FlirController<Network>{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        if let AfvCtlMessage::Network(msg) = msg{
            match msg{
                NetworkMessages::NalPacket(packet) => {
                    tokio::spawn(self.process_nal_packet(packet));
                }
                _ => {}
            }
            return;
        }

        
        if let AfvCtlMessage::Local(msg) = msg{
            match msg{
                LocalMessages::SelectedAfv(uuid) => {
                    *self.afv_uuid.write().await = uuid;
                },
                _ => {}
            }
            return;
        }
    }
    fn uuid(&self) -> BusUuid{
        self.bus_uuid
    }
}
#[async_trait]
impl BusElement<AfvCtlMessage> for FlirController<Local>{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        if let AfvCtlMessage::Local(msg) = msg{
            match msg{
                LocalMessages::SelectedAfv(uuid) => {
                    *self.afv_uuid.write().await = uuid;
                },
                LocalMessages::NalPacket(packet) => {
                    tokio::spawn(self.process_nal_packet(packet));
                }
                _ => {}
            }
            return;
        }
    }
    fn uuid(&self) -> BusUuid{
        self.bus_uuid
    }
}
