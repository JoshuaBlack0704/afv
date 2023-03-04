use std::sync::Arc;

use async_trait::async_trait;
use eframe::egui::Ui;
use rand::{thread_rng, Rng};
use tokio::{runtime::Handle, sync::RwLock};

use crate::{bus::{Bus, BusUuid, BusElement}, afvbus::AfvUuid, messages::{AfvCtlMessage, LocalMessages}};

use super::Renderable;

pub struct AfvController{
    uuid: BusUuid,
    afv_uuid: RwLock<AfvUuid>,
    bus: Bus<AfvCtlMessage>,
    handle: Handle,

    // Flir fields
}

impl AfvController{
    pub async fn new(bus: Bus<AfvCtlMessage>) -> Arc<AfvController> {
        let ctl = Arc::new(Self{
            uuid: thread_rng().gen(),
            bus: bus.clone(),
            handle: Handle::current(),
            afv_uuid: Default::default(),
        });

        bus.add_element(ctl.clone()).await;

        ctl
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
            }
            return;
        }
    }
    fn uuid(&self) -> BusUuid{
        self.uuid
    }
}

impl Renderable for AfvController{
    fn render(&self, ui: &mut Ui) {
        ui.label(self.afv_uuid.blocking_read().to_string());
    }
}