use std::sync::Arc;

use async_trait::async_trait;
use eframe::egui::{Ui, self};



use rand::{thread_rng, Rng};
use tokio::{runtime::Handle, sync::RwLock};

use crate::{bus::{Bus, BusUuid, BusElement}, afvbus::AfvUuid, messages::{AfvCtlMessage, LocalMessages}, flirops::FlirController, networkbus::Network, flirturret::FlirTurret, distancesensor::DistanceSensor, nozzleturret::NozzleTurret, targetops::TargetingComputer};

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

    flir: Arc<FlirController<Network>>,
    flir_turret: Arc<FlirTurret<Network>>,
    distance_sensor: Arc<DistanceSensor<Network>>,
    nozzle_turret: Arc<NozzleTurret<Network>>,
    targeting_comp: Arc<TargetingComputer<Network>>,
    
}

impl AfvController{
    pub async fn new(bus: Bus<AfvCtlMessage>) -> Arc<AfvController> {

        let flir = FlirController::<Network>::new(bus.clone()).await;
        let flir_turret = FlirTurret::<Network>::new().await;
        let nozzle_turret = NozzleTurret::<Network>::new().await;
        let distance_sensor = DistanceSensor::<Network>::new().await; 
        let targeting_comp = TargetingComputer::<Network>::new(bus.clone(), flir.clone(), flir_turret.clone(), nozzle_turret.clone(), distance_sensor.clone()).await;



        
        let ctl = Arc::new(Self{
            bus_uuid: thread_rng().gen(),
            bus: bus.clone(),
            handle: Handle::current(),
            afv_uuid: Default::default(),
            menu: RwLock::new(MenuTypes::Main),
            flir,
            flir_turret,
            distance_sensor,
            nozzle_turret,
            targeting_comp,
        });

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
            MenuTypes::FlirImageDisplay => self.flir.render_flir_display(ui),
        }
        
    }

    fn render_main(&self, ui: &mut Ui){
        self.targeting_comp.auto_target_button(ui);
        
    }
}

#[async_trait]
impl BusElement<AfvCtlMessage> for AfvController{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
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