#![allow(unused)]

use std::{sync::Arc, f64, f32};

use async_trait::async_trait;
use eframe::egui::{DragValue, plot::{Plot, Arrows}, Slider};
use serde::{Serialize, Deserialize};
use tokio::{runtime::{Handle, Runtime}, sync::RwLock, time::sleep};

use crate::{network::{ComEngine, AfvMessage, ComEngineService}, gui::GuiElement};

use super::flir::Flir;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TurretMsg{
    RequestAltitude,
    RequestAzimuth,
    RequestRate,
    RequestPumpState,
    Altitude(f32),
    Azimuth(f32),
    Rate(f32),
    PumpState(bool),
    Autotarget(bool),
}

#[async_trait]
pub trait Controller: Send + Sync{
    async fn altitude(&self) -> f32;
    async fn azimuth(&self) -> f32;
    async fn angular_rate(&self) -> f32;
    async fn pump_state(&self) -> bool;
    async fn set_altitude(&self, altitude: f32);
    async fn set_azimuth(&self, azimuth: f32);
    async fn set_angular_rate(&self, rate: f32);
    async fn set_pump_state(&self, state: bool);
    async fn refresh_autotarget(&self);
}

pub struct Turret{
    open: RwLock<bool>,
    pump_ctl: Arc<dyn Controller>,
    flir_ctl: Arc<dyn Controller>,
    handle: Handle,
    com: Option<Arc<ComEngine<AfvMessage>>>,
    flir: Arc<Flir>,

    // Manual control
    altitude: RwLock<f32>,
    azimuth: RwLock<f32>,
    angular_rate: RwLock<f32>,
    pump_state: RwLock<bool>,

    // Automatic control
    auto_target: RwLock<bool>,
    autopilot_timeout_budget: RwLock<u8>,
    auto_pump: RwLock<bool>,

    //Data control
    data_rate: RwLock<u8>,
    afv_altitude: RwLock<f32>,
    afv_azimuth: RwLock<f32>,
    afv_angular_rate: RwLock<f32>,
    afv_pump_state: RwLock<bool>,
}

pub struct PumpActuator{
    com: Option<Arc<ComEngine<AfvMessage>>>,
    
}
pub struct FlirActuator{
    com: Option<Arc<ComEngine<AfvMessage>>>,
    
}
pub struct Link{
    com: Arc<ComEngine<AfvMessage>>,
    altitude: RwLock<f32>,
    azimuth: RwLock<f32>,
    angular_rate: RwLock<f32>,
    pump_state: RwLock<bool>,
    auto_target: RwLock<bool>,
    auto_pump: RwLock<bool>,
}
pub struct Simulator{
    com: Option<Arc<ComEngine<AfvMessage>>>,
    altitude: RwLock<f32>,
    azimuth: RwLock<f32>,
    angular_rate: RwLock<f32>,
    pump_state: RwLock<bool>,
    auto_target: RwLock<bool>,
    auto_pump: RwLock<bool>,
    
    tgt_altitude: RwLock<f32>,
    tgt_azimuth: RwLock<f32>,
    
}



impl Turret{
    pub async fn actuated(com: Option<Arc<ComEngine<AfvMessage>>>, flir: Arc<Flir>) -> Arc<Turret> {
        let pump_ctl = PumpActuator::new(com.clone()).await;
        let flir_ctl = FlirActuator::new(com.clone()).await;
        Arc::new(Self{
            pump_ctl,
            handle: Handle::current(),
            com: None,
            flir,
            open: Default::default(),
            altitude: Default::default(),
            azimuth: Default::default(),
            angular_rate: Default::default(),
            auto_target: Default::default(),
            autopilot_timeout_budget: Default::default(),
            auto_pump: Default::default(),
            pump_state: Default::default(),
            data_rate: Default::default(),
            afv_altitude: Default::default(),
            afv_azimuth: Default::default(),
            afv_angular_rate: Default::default(),
            afv_pump_state: Default::default(),
            flir_ctl,
        })
        
    }
    pub fn actuated_blocking(rt: Arc<Runtime>, com: Option<Arc<ComEngine<AfvMessage>>>, flir: Arc<Flir>) -> Arc<Turret> {
        rt.block_on(Self::actuated(com, flir))
    }
    pub async fn linked(com: Arc<ComEngine<AfvMessage>>, flir: Arc<Flir>) -> Arc<Turret> {
        let pump_ctl = Link::new(com.clone()).await;
        let flir_ctl = Link::new(com.clone()).await;
        let turret = Arc::new(Self{
            pump_ctl,
            handle: Handle::current(),
            com: None,
            flir,
            open: Default::default(),
            altitude: Default::default(),
            azimuth: Default::default(),
            angular_rate: Default::default(),
            auto_target: Default::default(),
            autopilot_timeout_budget: Default::default(),
            auto_pump: Default::default(),
            pump_state: Default::default(),
            data_rate: RwLock::new(1),
            afv_altitude: Default::default(),
            afv_azimuth: Default::default(),
            afv_angular_rate: Default::default(),
            afv_pump_state: Default::default(),
            flir_ctl,
        });
        tokio::spawn(turret.clone().data_fetch());
        turret
    }
    pub fn linked_blocking(rt: Arc<Runtime>, com: Arc<ComEngine<AfvMessage>>, flir: Arc<Flir>) -> Arc<Turret> {
        rt.block_on(Self::linked(com, flir))
        
    }
    pub async fn simulated(com: Option<Arc<ComEngine<AfvMessage>>>, flir: Arc<Flir>) -> Arc<Turret> {
        let pump_ctl = Simulator::new(com.clone()).await;
        let flir_ctl = Simulator::new(com.clone()).await;
        Arc::new(Self{
            pump_ctl,
            handle: Handle::current(),
            com,
            flir,
            open: Default::default(),
            altitude: Default::default(),
            azimuth: Default::default(),
            angular_rate: Default::default(),
            auto_target: Default::default(),
            autopilot_timeout_budget: Default::default(),
            auto_pump: Default::default(),
            pump_state: Default::default(),
            data_rate: Default::default(),
            afv_altitude: Default::default(),
            afv_azimuth: Default::default(),
            afv_angular_rate: Default::default(),
            afv_pump_state: Default::default(),
            flir_ctl,
        })
        
    }
    pub fn simulated_blocking(rt: Arc<Runtime>, com: Option<Arc<ComEngine<AfvMessage>>>, flir: Arc<Flir>) -> Arc<Turret> {
        rt.block_on(Self::simulated(com, flir))
    }

    async fn data_fetch(self: Arc<Self>){
        loop{
            let sleep_time = (1.0/(*self.data_rate.read().await as f32) * 1000.0) as u64;
            sleep(tokio::time::Duration::from_millis(sleep_time)).await;
            *self.afv_altitude.write().await = self.pump_ctl.altitude().await;
            *self.afv_azimuth.write().await = self.pump_ctl.azimuth().await;
            *self.afv_angular_rate.write().await = self.pump_ctl.angular_rate().await;
            *self.afv_pump_state.write().await = self.pump_ctl.pump_state().await;
        }
    }
}

impl GuiElement for Turret{
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        self.open.blocking_write()
    }

    fn name(&self) -> String {
        "Turret Control".into()
    }

    fn render(self: Arc<Self>, ui: &mut eframe::egui::Ui) {
        let flir_image = self.flir.get_gui_image(ui);
        let size = ui.available_size();
        ui.horizontal_wrapped(|ui|{
            // Top left
            ui.allocate_ui(size/2.1, |ui|{
                ui.image(flir_image.id(), ui.available_size());
            });
            
            // Top Right
            ui.allocate_ui(size/2.1, |ui|{

                let azimuth = *self.afv_azimuth.blocking_read() as f64 + f64::consts::PI/2.0;

                let origin = [0.0,0.0];
                let target = [azimuth.cos() * 1.0, azimuth.sin() * 1.0];
                let azimuth_arrow = Arrows::new(origin, target).name("Azimuth arrow");
                
                
                Plot::new("Turret plot")
                .show_background(false)
                .data_aspect(1.0)
                .include_x(3.0)
                .include_y(3.0)
                .center_x_axis(true)
                .center_y_axis(true)
                .show_axes([false, false])
                .show(ui, |ui|{
                    ui.arrows(azimuth_arrow);
                });
            });
            
            // Bottom left
            ui.allocate_ui(size/2.1, |ui|{
                let mut pump_on = self.pump_state.blocking_write();
                
                ui.vertical_centered(|ui|{
                    ui.horizontal(|ui|{
                        let mut target_altitude = self.altitude.blocking_write();
                        ui.label("Tgt Altitude:  ");
                        let drag = DragValue::new(&mut (*target_altitude)).clamp_range(0..=90);
                        ui.add(drag);
                        
                    });
                    ui.horizontal(|ui|{
                        let mut target_azimuth = self.azimuth.blocking_write();
                        ui.label("Tgt Azimuth:  ");
                        let drag = DragValue::new(&mut (*target_azimuth)).clamp_range(-90..=90);
                        ui.add(drag);
                        
                    });
                    ui.horizontal(|ui|{
                        let mut angular_rate = self.angular_rate.blocking_write();
                        ui.label("Angular Rate:  ");
                        let drag = DragValue::new(&mut (*angular_rate)).clamp_range(0..=20);
                        ui.add(drag);
                        
                    });
                    if !*self.auto_target.blocking_read(){
                        if ui.button("Send commands").clicked(){
                            let altitude = *self.altitude.blocking_read() as f32 * 3.14/180.0;
                            let azimuth = *self.azimuth.blocking_read() as f32 * 3.14/180.0;
                            let angular_rate = *self.angular_rate.blocking_read() as f32 * 3.14/180.0;
                            self.handle.block_on(self.pump_ctl.set_altitude(altitude));
                            self.handle.block_on(self.pump_ctl.set_azimuth(azimuth));
                            self.handle.block_on(self.pump_ctl.set_angular_rate(angular_rate));
                        }
                    }
                    if !*self.auto_pump.blocking_read(){
                        if *pump_on{
                            if ui.button("Pump off").clicked(){
                                *pump_on = false;
                            }
                        }
                        else{
                            if ui.button("Pump on").clicked(){
                                *pump_on = true;
                            }
                        }
                    }
                });
            });
            
            // Bottom right
            ui.allocate_ui(size/2.1, |ui|{
                let mut auto_target = self.auto_target.blocking_write();
                let mut auto_pump = self.auto_pump.blocking_write();
                ui.vertical_centered_justified(|ui|{
                    if *auto_target{
                        if ui.button("Turn auto off").clicked(){
                            *auto_target = false;
                        }
                    }
                    else{
                        if ui.button("Turn auto target on").clicked(){
                            *auto_target = true;
                        }
                    }
                    if *auto_pump{
                        if ui.button("Turn auto pump off").clicked(){
                            *auto_pump = false;
                        }
                    }
                    else{
                        if ui.button("Turn auto pump on").clicked(){
                            *auto_pump = true;
                        }
                    }
                    ui.horizontal(|ui|{
                        let mut data_rate = self.data_rate.blocking_write();
                        let slide = Slider::new(&mut (*data_rate), 1..=20);
                        ui.label("Update rate");
                        ui.add(slide);
                    });
                });
            });
        });
    }
}


impl PumpActuator{
    pub async fn new(com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Self> {
        println!("Turret control established");
        Arc::new(Self{
            com,
        })
    }
}



#[async_trait]
impl Controller for PumpActuator{
    async fn altitude(&self) -> f32 {
        todo!()
    }

    async fn azimuth(&self) -> f32 {
        todo!()
    }

    async fn angular_rate(&self) -> f32 {
        todo!()
    }

    async fn pump_state(&self) -> bool{
        todo!()
    }

    async fn set_altitude(&self, altitude: f32) {
        todo!()
    }

    async fn set_azimuth(&self, azimuth: f32) {
        todo!()
    }

    async fn set_angular_rate(&self, rate: f32) {
        todo!()
    }

    async fn set_pump_state(&self, state: bool) {
        todo!()
    }

    async fn refresh_autotarget(&self) {
        todo!()
    }
}

#[async_trait]
impl Controller for FlirActuator{
    async fn altitude(&self) -> f32 {
        todo!()
    }

    async fn azimuth(&self) -> f32 {
        todo!()
    }

    async fn angular_rate(&self) -> f32 {
        todo!()
    }

    async fn pump_state(&self) -> bool{
        todo!()
    }

    async fn set_altitude(&self, altitude: f32) {
        todo!()
    }

    async fn set_azimuth(&self, azimuth: f32) {
        todo!()
    }

    async fn set_angular_rate(&self, rate: f32) {
        todo!()
    }

    async fn set_pump_state(&self, state: bool) {
        todo!()
    }

    async fn refresh_autotarget(&self) {
        todo!()
    }
}
impl FlirActuator{
    pub async fn new(com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Self> {
        println!("Turret control established");
        Arc::new(Self{
            com,
        })
    }
}

impl Link{
    pub async fn new(com: Arc<ComEngine<AfvMessage>>) -> Arc<Self> {
        println!("Turret link established");
        let link = Arc::new(Self{
            com: com.clone(),
            altitude: Default::default(),
            azimuth: Default::default(),
            angular_rate: Default::default(),
            pump_state: Default::default(),
            auto_target: Default::default(),
            auto_pump: Default::default(),
        });
        com.add_listener(link.clone()).await;
        link
    }
}

#[async_trait]
impl Controller for Link{
    async fn altitude(&self) -> f32 {
        self.com.send(AfvMessage::Turret(TurretMsg::RequestAltitude)).await;
        *self.altitude.read().await
    }

    async fn azimuth(&self) -> f32 {
        self.com.send(AfvMessage::Turret(TurretMsg::RequestAzimuth)).await;
        *self.azimuth.read().await
    }

    async fn angular_rate(&self) -> f32 {
        self.com.send(AfvMessage::Turret(TurretMsg::RequestRate)).await;
        *self.angular_rate.read().await
    }

    async fn pump_state(&self) -> bool {
        self.com.send(AfvMessage::Turret(TurretMsg::RequestPumpState)).await;
        *self.pump_state.read().await
    }

    async fn set_altitude(&self, altitude: f32) {
        self.com.send(AfvMessage::Turret(TurretMsg::Altitude(altitude))).await;
    }

    async fn set_azimuth(&self, azimuth: f32) {
        self.com.send(AfvMessage::Turret(TurretMsg::Azimuth(azimuth))).await;
    }

    async fn set_angular_rate(&self, rate: f32) {
        self.com.send(AfvMessage::Turret(TurretMsg::Rate(rate))).await;
    }

    async fn set_pump_state(&self, state: bool) {
        self.com.send(AfvMessage::Turret(TurretMsg::PumpState(state))).await;
    }

    async fn refresh_autotarget(&self) {
        self.com.send(AfvMessage::Turret(TurretMsg::Autotarget(*self.auto_target.read().await))).await;
    }
}

#[async_trait]
impl ComEngineService<AfvMessage> for Link{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::Turret(msg) = msg{
            match msg{
                TurretMsg::Altitude(alt) => *self.altitude.write().await = alt,
                TurretMsg::Azimuth(azi) =>  *self.azimuth.write().await = azi,
                TurretMsg::Rate(rate) => *self.angular_rate.write().await = rate,
                TurretMsg::PumpState(state) => *self.pump_state.write().await = state,
                _ => {}
            };
            return;
        }
        
    }
}

impl Simulator{
    pub async fn new(com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Self> {
        let sim = Arc::new(Self{
            com: com.clone(),
            altitude: Default::default(),
            azimuth: Default::default(),
            angular_rate: Default::default(),
            pump_state: Default::default(),
            auto_target: Default::default(),
            auto_pump: Default::default(),
            tgt_altitude: Default::default(),
            tgt_azimuth: Default::default(),
        });
        if let Some(com) = com{
            com.add_listener(sim.clone()).await;
            println!("Turret simulation established");
        }
        tokio::spawn(sim.clone().simulation());
        sim
    }
    async fn simulation(self: Arc<Self>){
        let time_step_millis = 10;
        let time_step_secs = time_step_millis as f32/1000.0;
        loop{
            let tgt_azimuth = -*self.tgt_azimuth.read().await;
            let rate = *self.angular_rate.read().await;
            let angular_dist = rate * time_step_secs;
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
    async fn altitude(&self) -> f32 {
        *self.altitude.read().await
    }

    async fn azimuth(&self) -> f32 {
        *self.azimuth.read().await
    }

    async fn angular_rate(&self) -> f32 {
        *self.angular_rate.read().await
    }

    async fn pump_state(&self) -> bool {
        *self.pump_state.read().await
    }

    async fn set_altitude(&self, altitude: f32) {
        *self.altitude.write().await = altitude;
    }

    async fn set_azimuth(&self, azimuth: f32) {
        *self.azimuth.write().await = azimuth;
    }

    async fn set_angular_rate(&self, rate: f32) {
        *self.angular_rate.write().await = rate;
    }

    async fn set_pump_state(&self, state: bool) {
        *self.pump_state.write().await = state;
    }

    async fn refresh_autotarget(&self) {
        todo!()
    }
}

#[async_trait]
impl ComEngineService<AfvMessage> for Simulator{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::Turret(msg) = msg{
            match msg{
                TurretMsg::Altitude(alt) => *self.tgt_altitude.write().await = alt,
                TurretMsg::Azimuth(azi) => *self.tgt_azimuth.write().await = azi,
                TurretMsg::Rate(rate) => *self.angular_rate.write().await = rate,
                TurretMsg::PumpState(state) => *self.pump_state.write().await = state,
                TurretMsg::RequestAltitude => {
                    if let Some(com) = &self.com{
                        com.send(AfvMessage::Turret(TurretMsg::Altitude(*self.altitude.read().await))).await;
                    }
                },
                TurretMsg::RequestAzimuth => {
                    if let Some(com) = &self.com{
                        com.send(AfvMessage::Turret(TurretMsg::Azimuth(*self.azimuth.read().await))).await;
                    }
                },
                TurretMsg::RequestRate => {
                    if let Some(com) = &self.com{
                        com.send(AfvMessage::Turret(TurretMsg::Rate(*self.angular_rate.read().await))).await;
                    }
                },
                TurretMsg::RequestPumpState => {
                    if let Some(com) = &self.com{
                        com.send(AfvMessage::Turret(TurretMsg::PumpState(*self.pump_state.read().await))).await;
                    }
                },
                TurretMsg::Autotarget(_) => todo!(),
            };
            return;
        }
        
    }
}