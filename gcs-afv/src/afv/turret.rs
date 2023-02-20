#![allow(unused)]
use std::{sync::Arc, f64::consts::PI};

use async_trait::async_trait;
use eframe::egui::{Ui, ComboBox, DragValue, plot::{Arrows, Plot}};
use serde::{Deserialize, Serialize};
use tokio::{sync::RwLock, runtime::Handle, time::{Duration, sleep}};

use crate::{network::{ComEngine, AfvMessage, ComEngineService}, gui::GuiElement};

use super::flir::Flir;

#[async_trait]
pub trait Controller: Send + Sync{
    async fn get_altitude(&self) -> f32;
    async fn get_azimuth(&self) -> f32;
    async fn set_altitude(&self, altitude: f32);
    async fn set_azimuth(&self, azimuth: f32);
}

type FlirCtl = (Arc<Flir>, Arc<dyn Controller>);
type Com = Arc<ComEngine<AfvMessage>>;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Copy)]
pub struct TurretCode(u32);
impl TurretCode{
    const TOPFLIRCODE: TurretCode = TurretCode(1);
    const NOZZLECODE: TurretCode = TurretCode(2);
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TurretMsg{
    GetAltitude(TurretCode),
    GetAzimuth(TurretCode),
    
    SetAltitude(TurretCode, f32),
    SetAzimuth(TurretCode, f32),
}

pub struct Turret{
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
    /// The const code this link controls
    code: TurretCode,
    
    altitude: RwLock<f32>,
    azimuth: RwLock<f32>,
    tgt_altitude: RwLock<f32>,
    tgt_azimuth: RwLock<f32>,
}

pub struct Simulator{
    com: Option<Com>,
    /// The const code this link controls
    code: TurretCode,
    
    altitude: RwLock<f32>,
    azimuth: RwLock<f32>,
    tgt_altitude: RwLock<f32>,
    tgt_azimuth: RwLock<f32>,
}

impl Turret{
    pub async fn linked(com: Com, top_flir: Arc<Flir>) -> Arc<Self> {
        let nozzle_link = Link::new(TurretCode::NOZZLECODE, com.clone()).await;
        let top_flir_link = Link::new(TurretCode::TOPFLIRCODE, com.clone()).await;
        let turret = Arc::new(Self{
            open: Default::default(),
            handle: Handle::current(),
            com: Some(com.clone()),
            nozzle_ctl: nozzle_link.clone(),
            flir_ctls: vec![(top_flir.clone(), top_flir_link.clone())],
            active_flir: Default::default(),
            altitude: Default::default(),
            azimuth: Default::default(),
            data_rate: Default::default(),
        });

        com.add_listener(turret.clone()).await;

        turret
        
    }
    pub async fn simulated(com: Option<Com>, top_flir: Arc<Flir>) -> Arc<Self> {
        let nozzle_link = Simulator::new(TurretCode::NOZZLECODE, com.clone()).await;
        let top_flir_link = Simulator::new(TurretCode::TOPFLIRCODE, com.clone()).await;
        let turret = Arc::new(Self{
            open: Default::default(),
            handle: Handle::current(),
            com: com.clone(),
            nozzle_ctl: nozzle_link.clone(),
            flir_ctls: vec![(top_flir.clone(), top_flir_link.clone())],
            active_flir: Default::default(),
            altitude: Default::default(),
            azimuth: Default::default(),
            data_rate: Default::default(),
        });

        if let Some(c) = com{
            c.add_listener(turret.clone()).await;
        }

        turret
        
    }
    
    fn flir_image_ui(&self, ui: &mut Ui){
        let texture = self.flir_ctls[*self.active_flir.blocking_read()].0.get_gui_image(ui);
        ui.image(texture.id(), ui.available_size());
        
    }
    fn nav_ui(&self, ui: &mut Ui){
        let nozzle_azimuth = self.handle.block_on(self.nozzle_ctl.get_azimuth()) as f64 + PI/2.0;
        let nozzle_azimuth = [nozzle_azimuth.cos(), nozzle_azimuth.sin()];
        let flir_azimuth = (self.handle.block_on(self.flir_ctls[*self.active_flir.blocking_read()].1.get_azimuth()) as f64) + PI/2.0;
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

            ui.label(format!("Current Altitude: {:.2} Current Azimuth: {:.2}", current_altitude, current_azimuth));
            let input_alt_drag = DragValue::new(&mut (*input_altitude)).clamp_range(-90..=90);
            ui.add(input_alt_drag);
            if ui.button("Set Target Altitude").clicked(){
                let input_altitude = *input_altitude * 3.14/180.0;
                self.handle.block_on(active.set_altitude(input_altitude));
            }
            let input_azi_drag = DragValue::new(&mut (*input_azimuth)).clamp_range(-90..=90);
            ui.add(input_azi_drag);
            if ui.button("Set Target Azimuth").clicked(){
                let input_azimuth = *input_azimuth * 3.14/180.0;
                self.handle.block_on(active.set_azimuth(input_azimuth));
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

            ui.label(format!("Current Altitude: {:.2} Current Azimuth: {:.2}", current_altitude, current_azimuth));
            let input_alt_drag = DragValue::new(&mut (*input_altitude)).clamp_range(-90..=90);
            ui.add(input_alt_drag);
            if ui.button("Set Target Altitude").clicked(){
                let input_altitude = *input_altitude * 3.14/180.0;
                self.handle.block_on(nozzle.set_altitude(input_altitude));
            }
            let input_azi_drag = DragValue::new(&mut (*input_azimuth)).clamp_range(-90..=90);
            ui.add(input_azi_drag);
            if ui.button("Set Target Azimuth").clicked(){
                let input_azimuth = *input_azimuth * 3.14/180.0;
                self.handle.block_on(nozzle.set_azimuth(input_azimuth));
            }
            
            
        });
        
    }
}

impl GuiElement for Turret{
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
impl ComEngineService<AfvMessage> for Turret{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
    }
}

impl Link{
    pub async fn new(code: TurretCode, com: Arc<ComEngine<AfvMessage>>) -> Arc<Self> {
        let link = Arc::new(Self{
            com: com.clone(),
            altitude: Default::default(),
            azimuth: Default::default(),
            tgt_altitude: Default::default(),
            tgt_azimuth: Default::default(),
            code,
        });

        com.add_listener(link.clone()).await;

        tokio::spawn(link.clone().update());
        
        link
    }

    async fn update(self: Arc<Self>){
        let wait = Duration::from_millis(500);
        loop{
            sleep(wait).await;
            self.com.send(AfvMessage::Turret(TurretMsg::GetAltitude(self.code))).await;
            self.com.send(AfvMessage::Turret(TurretMsg::GetAzimuth(self.code))).await;
        }
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
        self.com.send(AfvMessage::Turret(TurretMsg::SetAltitude(self.code, altitude))).await;
    }

    async fn set_azimuth(&self, azimuth: f32) {
        self.com.send(AfvMessage::Turret(TurretMsg::SetAzimuth(self.code, azimuth))).await;
    }
}

#[async_trait]
impl ComEngineService<AfvMessage> for Link{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::Turret(m) = msg{
            match m{
                TurretMsg::SetAltitude(code, alt) => {
                    if code == self.code{
                        *self.altitude.write().await = alt;
                    }
                },
                TurretMsg::SetAzimuth(code, azi) => {
                    if code == self.code{
                        *self.azimuth.write().await = azi;
                    }
                },
                _ => {}
            }
        }
    }
}


impl Simulator{
    pub async fn new(code: TurretCode, com: Option<Com>) -> Arc<Self> {
        let sim = Arc::new(Self{
            com: com.clone(),
            code,
            altitude: Default::default(),
            azimuth: Default::default(),
            tgt_altitude: Default::default(),
            tgt_azimuth: Default::default(),
        });

        if let Some(c) = com{
            c.add_listener(sim.clone()).await;
        }

        tokio::spawn(sim.clone().simulate());

        sim
    }

    async fn simulate(self: Arc<Self>){
        let time_step_millis = 10;
        let time_step_secs = time_step_millis as f32/1000.0;
        let angular_rate = std::f32::consts::PI/2.0;
        loop{
            let tgt_azimuth = -*self.tgt_azimuth.read().await;
            let angular_dist = angular_rate * time_step_secs;
            // println!("{}", angular_dist);
            
            let mut azimuth = self.azimuth.write().await;
            let dif = tgt_azimuth - *azimuth;

            if dif.abs() > angular_dist.abs(){
                *azimuth += angular_dist * dif.signum();
            }
            else{
                *azimuth = tgt_azimuth;
            }
            sleep(tokio::time::Duration::from_millis(time_step_millis)).await;
        }
    }
}

#[async_trait]
impl Controller for Simulator{
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
impl ComEngineService<AfvMessage> for Simulator{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::Turret(m) = msg{
            match m{
                TurretMsg::GetAltitude(code) => {
                    if let Some(c) = &self.com{
                        c.send(AfvMessage::Turret(TurretMsg::SetAltitude(self.code, self.get_altitude().await))).await;
                    }
                },
                TurretMsg::GetAzimuth(code) => {
                    if let Some(c) = &self.com{
                        c.send(AfvMessage::Turret(TurretMsg::SetAzimuth(self.code, self.get_azimuth().await))).await;
                    }
                },
                TurretMsg::SetAltitude(code, alt) => {
                    if code == self.code{
                        println!("Here");
                        self.set_altitude(alt).await;
                    }
                },
                TurretMsg::SetAzimuth(code, azi) => {
                    if code == self.code{
                        self.set_azimuth(azi).await;
                    }
                },
            }
        }
        
    }
}
