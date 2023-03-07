use std::sync::Arc;

use async_trait::async_trait;
use eframe::{egui::{Ui, self}, epaint::TextureHandle};
use image::{DynamicImage, ImageBuffer};
use openh264::{decoder::Decoder, to_bitstream_with_001_be, nal_units};
use rand::{thread_rng, Rng};
use tokio::{runtime::Handle, sync::RwLock, time::sleep};

use crate::{bus::{Bus, BusUuid, BusElement}, afvbus::AfvUuid, messages::{AfvCtlMessage, LocalMessages, NetworkMessages}};

use super::Renderable;

#[derive(PartialEq, Eq)]
enum MenuTypes{
    Main,
    FlirImageDisplay,
}

pub struct AfvController{
    bus_uuid: BusUuid,
    afv_uuid: RwLock<AfvUuid>,
    bus: Bus<AfvCtlMessage>,
    handle: Handle,

    //Current menu
    menu: RwLock<MenuTypes>,

    // Flir fields
    flir_stream: RwLock<bool>,
    flir_decoder: RwLock<Option<Decoder>>,
    flir_image: RwLock<DynamicImage>,
    gui_image: RwLock<Option<TextureHandle>>,
}

impl AfvController{
    pub async fn new(bus: Bus<AfvCtlMessage>) -> Arc<AfvController> {
        let ctl = Arc::new(Self{
            bus_uuid: thread_rng().gen(),
            bus: bus.clone(),
            handle: Handle::current(),
            afv_uuid: Default::default(),
            flir_stream: Default::default(),
            flir_image: RwLock::new(DynamicImage::new_rgb8(300,300)),
            gui_image: Default::default(),
            menu: RwLock::new(MenuTypes::Main),
            flir_decoder: Default::default(),
        });

        tokio::spawn(ctl.clone().flir_stream_manager());

        bus.add_element(ctl.clone()).await;

        ctl
    }
    fn get_gui_image(&self, ui: &mut Ui) -> TextureHandle{
        let mut gui_image = self.gui_image.blocking_write();
        if let Some(i) = &(*gui_image){
            return i.clone();
        }

        let image = self.flir_image.blocking_read();
        let rgb = image.as_rgb8().unwrap();
        let pixels = rgb.as_flat_samples();
        let size = [rgb.width() as usize, rgb.height() as usize];
        let color_image = egui::ColorImage::from_rgb(size, pixels.as_slice());
        let handle = ui.ctx().load_texture("Flir Output", color_image, Default::default());

        *gui_image = Some(handle.clone());
        handle
    }
    fn left_panel(&self, ui: &mut Ui){
        let mut menu = self.menu.blocking_write();

        ui.selectable_value(&mut (*menu), MenuTypes::Main, "Main Control");
        ui.selectable_value(&mut (*menu), MenuTypes::FlirImageDisplay, "Flir Display");
    }
    fn central_panel(&self, ui: &mut Ui){
        match *self.menu.blocking_read(){
            MenuTypes::Main => self.render_main(ui),
            MenuTypes::FlirImageDisplay => self.render_flir_display(ui),
        }
        
    }

    fn render_main(&self, _ui: &mut Ui){
        
    }
    fn render_flir_display(&self, ui: &mut Ui){
        ui.horizontal(|ui|{
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
        });
        let texture = self.get_gui_image(ui);
        ui.image(texture.id(), ui.available_size());
    }
    async fn process_nal_packet(self: Arc<Self>, packet: Vec<u8>){
        let mut nal = Vec::with_capacity(packet.len());
        to_bitstream_with_001_be::<u32>(&packet, &mut nal);
        for packet in nal_units(&nal){
            if let Some(d) = &mut (*self.flir_decoder.write().await){
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
                    *self.flir_image.write().await = DynamicImage::ImageRgb8(image_data.clone());
                    *self.gui_image.write().await = None;
                }
            }
        }
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

#[async_trait]
impl BusElement<AfvCtlMessage> for AfvController{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        if let AfvCtlMessage::Network(msg) = msg{
            match msg{
                NetworkMessages::NalPacket(_, packet) => {
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

impl Renderable for AfvController{
    fn render(&self, ui: &mut Ui) {
        egui::SidePanel::left("Ctl menu").show_inside(ui, |ui|{
            self.left_panel(ui);
        });
        egui::CentralPanel::default().show_inside(ui, |ui|{
            self.central_panel(ui);
        });
    }
}