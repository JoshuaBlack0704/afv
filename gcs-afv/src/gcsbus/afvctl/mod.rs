use std::sync::Arc;

use async_trait::async_trait;
use eframe::{egui::{Ui, self}, epaint::TextureHandle};
use glam::Vec2;
use image::DynamicImage;
use openh264::decoder::Decoder;
use rand::{thread_rng, Rng};
use tokio::{runtime::Handle, sync::RwLock};

use crate::{bus::{Bus, BusUuid, BusElement}, afvbus::AfvUuid, messages::{AfvCtlMessage, LocalMessages, NetworkMessages}};

use super::Renderable;

mod flir;

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

impl AfvController{
    pub async fn new(bus: Bus<AfvCtlMessage>) -> Arc<AfvController> {
        let ctl = Arc::new(Self{
            bus_uuid: thread_rng().gen(),
            bus: bus.clone(),
            handle: Handle::current(),
            afv_uuid: Default::default(),
            flir_image: RwLock::new(DynamicImage::new_rgb8(300,300)),
            flir_gui_image: Default::default(),
            menu: RwLock::new(MenuTypes::Main),
            flir_decoder: Default::default(),
            flir_filtered_image: Default::default(),
            flir_filter: Default::default(),
            flir_filter_level: Default::default(),
            flir_analysis_barrier: Default::default(),
            flir_centroids: Default::default(),
            flir_target_iterations: RwLock::new(2),
        });

        tokio::spawn(ctl.clone().flir_stream_manager());

        bus.add_element(ctl.clone()).await;

        ctl
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
}

#[async_trait]
impl BusElement<AfvCtlMessage> for AfvController{
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