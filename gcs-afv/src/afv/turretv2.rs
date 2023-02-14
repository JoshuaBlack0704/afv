#![allow(unused)]
use std::sync::Arc;

use async_trait::async_trait;
use eframe::egui::{Ui, ComboBox};
use tokio::{sync::RwLock, runtime::Handle};

use crate::{network::{ComEngine, AfvMessage, ComEngineService}, gui::GuiElement};

use super::flir::Flir;

pub trait Controller: Send + Sync{
    
}

type FlirCtl = (Arc<Flir>, Arc<dyn Controller>);

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

            
            
        });
    }
    fn nozzle_ctl_ui(&self, ui: &mut Ui){
        
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

