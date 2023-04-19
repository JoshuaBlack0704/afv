use std::sync::Arc;

use eframe::egui::{TextureHandle, DragValue, self, Ui};
use image::{DynamicImage, ImageBuffer};
use log::info;
use openh264::{decoder::Decoder, to_bitstream_with_001_be, nal_units};
use tokio::{sync::{broadcast, RwLock}, time::{sleep, Duration}, runtime::Handle};

use crate::{network::NetMessage, operators::flir::{FlirOperatorSettings, FlirOperatorMessage}, ui::Renderable, drivers::flir::FlirDriverMessage};

#[derive(Clone)]
pub struct FlirSystemCommunicator{
    handle: Handle,
    tx: broadcast::Sender<NetMessage>,

    settings: Option<FlirOperatorSettings>,

    latest_image: Arc<RwLock<DynamicImage>>,
    latest_ui_image: Arc<RwLock<Option<TextureHandle>>>,
    stream: Arc<RwLock<Option<Decoder>>>,
}

impl FlirSystemCommunicator{
    pub async fn new(tx: broadcast::Sender<NetMessage>) -> FlirSystemCommunicator {
        let comm = Self{
            tx,
            settings: Default::default(),
            latest_image: Default::default(),
            latest_ui_image: Default::default(),
            stream: Default::default(),
            handle: Handle::current(),
        };

        tokio::spawn(comm.clone().start());
        tokio::spawn(comm.clone().stream_handler());

        comm
    }
    async fn start(self){
        info!("Flir system starting");
        let mut rx = self.tx.subscribe();
        loop{
            let msg = match rx.recv().await{
                Ok(msg) => msg,
                Err(_) => continue,
            };
        }
    }
    pub async fn latest_ui_image(&self, ui: &mut Ui) -> TextureHandle {
        let mut gui_image = self.latest_ui_image.write().await;
        if let Some(i) = &(*gui_image){
            return i.clone();
        }

        let image = self.latest_image.read().await.clone().into_rgb8();
        // Select processed or direct image
        let pixels = image.as_flat_samples();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = egui::ColorImage::from_rgb(size, pixels.as_slice());
        let handle = ui.ctx().load_texture("Flir Output", color_image, Default::default());

        *gui_image = Some(handle.clone());
        handle        
    }
    async fn poll_stream(self, msg: FlirDriverMessage){
        *self.stream.write().await = Some(Decoder::new().unwrap());
        while let Some(_) = *self.stream.read().await{
            sleep(Duration::from_secs(3)).await;
            let _ = self.tx.send(NetMessage::FlirDriver(msg.clone()));
        }
    }
    async fn stream_handler(self){
        let mut rx = self.tx.subscribe();
        loop{
            if let Ok(NetMessage::FlirDriver(FlirDriverMessage::NalPacket(data))) = rx.recv().await{
                if let Some(decoder) = &mut(*self.stream.write().await){
                    let mut nal = data.clone();
                    to_bitstream_with_001_be::<u32>(&data, &mut nal);
                    for packet in nal_units(&nal){
                        if let Ok(Some(yuv)) = decoder.decode(&packet){
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
                                *self.latest_image.write().await = DynamicImage::ImageRgb8(image_data.clone());
                                *self.latest_ui_image.write().await = None;
                            }
                        }                    
                    }
                }
            }
        } 
    }
    fn plot_image(&mut self){
        
        
        
    }
}

impl Renderable for FlirSystemCommunicator{
    fn render(&mut self, ui: &mut eframe::egui::Ui) {
        ui.vertical_centered(|ui|{
            ui.horizontal(|ui|{
                if ui.button("Poll Settings").clicked(){
                    self.settings = None;
                }
                if let Some(settings) = &mut self.settings{
                    if ui.button("Send settings").clicked(){
                       let _ = self.tx.send(NetMessage::FlirOperator(FlirOperatorMessage::Settings(settings.clone())));
                    }
                    let drag = DragValue::new(&mut settings.fliter_value).speed(1).clamp_range(0..=255);
                    ui.add(drag);
                }
                let mut decoder = self.stream.blocking_write();
                match &*decoder{
                    Some(_) => {
                        if ui.button("Stop Streaming").clicked(){
                            *decoder = None;
                        }
                    },
                    None => {
                        if ui.button("Stream Ir").clicked(){
                            self.handle.spawn(self.clone().poll_stream(FlirDriverMessage::OpenIrStream));
                        }
                        if ui.button("Stream Visual").clicked(){
                            self.handle.spawn(self.clone().poll_stream(FlirDriverMessage::OpenIrStream));
                        }
                    },
                }
                drop(decoder);
                
                let texture = self.handle.block_on(self.latest_ui_image(ui));
              ui.image(texture.id(), ui.available_size());  
            });
        });
    }
}