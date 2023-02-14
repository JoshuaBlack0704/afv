#![allow(unused)]
use std::sync::Arc;

use async_trait::async_trait;
use eframe::egui::{Ui, ComboBox, DragValue, plot::{Arrows, Plot}};
use tokio::{sync::RwLock, runtime::Handle};

use crate::{network::{ComEngine, AfvMessage, ComEngineService}, gui::GuiElement};

use super::flir::Flir;

const TOPFLIRCODE:u32 = 1;
const NOZZLECODE:u32 = 2;

#[async_trait]
pub trait Controller: Send + Sync{
    async fn get_altitude(&self) -> f32;
    async fn get_azimuth(&self) -> f32;
    async fn set_altitude(&self, altitude: f32);
    async fn set_azimuth(&self, azimuth: f32);
}

type FlirCtl = (Arc<Flir>, Arc<dyn Controller>);
type Com = Arc<ComEngine<AfvMessage>>;

pub struct Turret2{
    /// The open selector for guis
    open: RwLock<bool>,
    /// The runtime handle
    handle: Handle,
    /// The com system
    com: Option<Arc<ComEngine<AfvMessage>>>,
    
    /// The controller handling the nozzle turret
    nozzle_ctl: Arc<dyn Controller>,
    /// The controllers handling the flir turrets
    flir_ctls: Vec<FlirCtl>,
    /// The active flir
    active_flir: RwLock<usize>,

    /// The altitude input
    altitude: RwLock<f32>,
    /// The azimuth input
    azimuth: RwLock<f32>,

    /// The data request rate
    data_rate: RwLock<u8>,
}

pub struct Link{
    com: Com,
    
    altitude: RwLock<f32>,
    azimuth: RwLock<f32>,
    tgt_altitude: RwLock<f32>,
    tgt_azimuth: RwLock<f32>,
}

pub struct Simulator{
    com: Option<Com>,
    
    altitude: RwLock<f32>,
    azimuth: RwLock<f32>,
    tgt_altitude: RwLock<f32>,
    tgt_azimuth: RwLock<f32>,
}

impl Turret2{
    pub async fn new(nozzle_ctl: Arc<dyn Controller>, flirs: &[FlirCtl], com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Self> {
        let turret = Arc::new(Self{
            open: Default::default(),
            handle: Handle::current(),
            com: com.clone(),
            nozzle_ctl,
            flir_ctls: flirs.to_vec(),
            altitude: Default::default(),
            azimuth: Default::default(),
            data_rate: Default::default(),
            active_flir: Default::default(),
        });

        if let Some(c) = com{
            c.add_listener(turret.clone());
        }

        turret
    }
    fn flir_image_ui(&self, ui: &mut Ui){
        let texture = self.flir_ctls[*self.active_flir.blocking_read()].0.get_gui_image(ui);
        ui.image(texture.id(), ui.available_size());
        
    }
    fn nav_ui(&self, ui: &mut Ui){
        let nozzle_azimuth = self.handle.block_on(self.nozzle_ctl.get_azimuth()) as f64;
        let nozzle_azimuth = [nozzle_azimuth.cos(), nozzle_azimuth.sin()];
        let flir_azimuth = self.handle.block_on(self.flir_ctls[*self.active_flir.blocking_read()].1.get_azimuth()) as f64;
        let flir_azimuth = [flir_azimuth.cos(), flir_azimuth.sin()];

        let origin = [0.0,0.0];

        let arrows = Arrows::new([origin,origin].to_vec(), [nozzle_azimuth, flir_azimuth].to_vec()).name("Targeting arrows");
        
        Plot::new("Turret plot")
        .show_background(false)
        .data_aspect(1.0)
        .include_x(3.0)
        .include_y(3.0)
        .center_x_axis(true)
        .center_y_axis(true)
        .show_axes([false, false])
        .show(ui, |ui|{
            ui.arrows(arrows);
        });
    }
    fn flir_ctl_ui(&self, ui: &mut Ui){
        let mut active = self.active_flir.blocking_write();
        ui.vertical_centered_justified(|ui|{
            ComboBox::from_label("Flir selection")
            .selected_text(format!("FLIR #{}", *active))
            .show_ui(ui, |ui|{
                for (index ,flir) in self.flir_ctls.iter().enumerate(){
                    ui.selectable_value(&mut (*active), index, format!("Flir #{}", index));    
                }
            });
            ui.separator();

            let active = self.flir_ctls[*active].1.clone();
            let current_altitude = self.handle.block_on(active.get_altitude());
            let current_azimuth = self.handle.block_on(active.get_azimuth());

            let mut input_altitude = self.altitude.blocking_write();
            let mut input_azimuth = self.azimuth.blocking_write();

            ui.label(format!("Current Alitude: {:.2} Current Azimuth: {:.2}", current_altitude, current_azimuth));
            let input_alt_drag = DragValue::new(&mut (*input_altitude)).clamp_range(-90..=90);
            ui.add(input_alt_drag);
            if ui.button("Set Target Alitude").clicked(){
                self.handle.block_on(active.set_altitude(*input_altitude));
            }
            let input_azi_drag = DragValue::new(&mut (*input_azimuth)).clamp_range(-90..=90);
            ui.add(input_azi_drag);
            if ui.button("Set Target Alitude").clicked(){
                self.handle.block_on(active.set_azimuth(*input_azimuth));
            }
        });
    }
    fn nozzle_ctl_ui(&self, ui: &mut Ui){
        ui.vertical_centered_justified(|ui|{
            let nozzle = self.nozzle_ctl.clone();
            let current_altitude = self.handle.block_on(nozzle.get_altitude());
            let current_azimuth = self.handle.block_on(nozzle.get_azimuth());

            let mut input_altitude = self.altitude.blocking_write();
            let mut input_azimuth = self.azimuth.blocking_write();

            ui.label(format!("Current Alitude: {:.2} Current Azimuth: {:.2}", current_altitude, current_azimuth));
            let input_alt_drag = DragValue::new(&mut (*input_altitude)).clamp_range(-90..=90);
            ui.add(input_alt_drag);
            if ui.button("Set Target Alitude").clicked(){
                self.handle.block_on(nozzle.set_altitude(*input_altitude));
            }
            let input_azi_drag = DragValue::new(&mut (*input_azimuth)).clamp_range(-90..=90);
            ui.add(input_azi_drag);
            if ui.button("Set Target Alitude").clicked(){
                self.handle.block_on(nozzle.set_azimuth(*input_azimuth));
            }
            
            
        });
        
    }
}

impl GuiElement for Turret2{
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        self.open.blocking_write()
    }

    fn name(&self) -> String {
        "Turret System".into()
    }

    fn render(self: Arc<Self>, ui: &mut eframe::egui::Ui) {
        let size = ui.available_size();
        ui.horizontal_wrapped(|ui|{
            // Top left
            ui.allocate_ui(size/2.1, |ui|{
                self.flir_image_ui(ui);
            });
            
            // Top Right
            ui.allocate_ui(size/2.1, |ui|{
                self.nav_ui(ui);
            });
            
            // Bottom left
            ui.allocate_ui(size/2.1, |ui|{
                self.flir_ctl_ui(ui);
            });
            
            // Bottom right
            ui.allocate_ui(size/2.1, |ui|{
                self.nozzle_ctl_ui(ui);
            });
        });
    }
}

#[async_trait]
impl ComEngineService<AfvMessage> for Turret2{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        
        todo!()
    }
}

impl Link{
    pub async fn new(com: Arc<ComEngine<AfvMessage>>) -> Arc<Self> {
        let link = Arc::new(Self{
            com: com.clone(),
            altitude: Default::default(),
            azimuth: Default::default(),
            tgt_altitude: Default::default(),
            tgt_azimuth: Default::default(),
        });

        com.add_listener(link.clone()).await;

        link
    }
}

#[async_trait]
impl Controller for Link{
    async fn get_altitude(&self) -> f32 {
        *self.altitude.read().await
    }

    async fn get_azimuth(&self) -> f32 {
        *self.azimuth.read().await
    }

    async fn set_altitude(&self, altitude: f32) {
        *self.tgt_altitude.write().await = altitude;
    }

    async fn set_azimuth(&self, azimuth: f32) {
        *self.tgt_azimuth.write().await = azimuth;
    }
}

#[async_trait]
impl ComEngineService<AfvMessage> for Link{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        
    }
}


impl Simulator{
    pub fn new()
}

#[async_trait]
impl ComEngineService<AfvMessage> for Simulator{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        
    }
}
