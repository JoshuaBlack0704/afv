use std::sync::Arc;

use async_trait::async_trait;
use eframe::egui::{Ui, self};



use rand::{thread_rng, Rng};
use tokio::{runtime::Handle, sync::RwLock};

use crate::{bus::{Bus, BusUuid, BusElement}, afvbus::AfvUuid, messages::{AfvCtlMessage, LocalMessages}, flirops::{FlirController, Network}};

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

    // Flir
    flir: Arc<FlirController<Network>>,
}

impl AfvController{
    pub async fn new(bus: Bus<AfvCtlMessage>) -> Arc<AfvController> {
        let ctl = Arc::new(Self{
            bus_uuid: thread_rng().gen(),
            bus: bus.clone(),
            handle: Handle::current(),
            afv_uuid: Default::default(),
            menu: RwLock::new(MenuTypes::Main),
            flir: FlirController::<Network>::new(bus.clone()).await,
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

    fn render_main(&self, _ui: &mut Ui){
        
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