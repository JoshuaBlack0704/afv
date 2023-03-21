use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use eframe::egui::ComboBox;
use rand::{thread_rng, Rng};
use tokio::{sync::RwLock, runtime::Handle};

use crate::{bus::{Bus, BusElement, BusUuid}, afv::AfvUuid, messages::{AfvCtlMessage, NetworkMessages, LocalMessages}};

use super::Renderable;

pub struct AfvSelector{
    uuid: BusUuid,
    afv: RwLock<HashSet<AfvUuid>>,
    active: RwLock<AfvUuid>,
    handle: Handle,
    bus: Bus<AfvCtlMessage>,
}

impl AfvSelector{
    pub async fn new(bus: Bus<AfvCtlMessage>) -> Arc<AfvSelector> {
        let selector = Arc::new(Self{
            uuid: thread_rng().gen(),
            afv: Default::default(),
            active: Default::default(),
            bus: bus.clone(),
            handle: Handle::current(),
        });

        bus.add_element(selector.clone()).await;

        selector
    }

    async fn add_afv(self: Arc<Self>, uuid: AfvUuid){
        let mut set = self.afv.write().await;
        if let None = set.get(&uuid){
            set.insert(uuid);
        }
    }
}

#[async_trait]
impl BusElement<AfvCtlMessage> for AfvSelector{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        if let AfvCtlMessage::Network(msg) = msg{
            if let NetworkMessages::AfvUuid(uuid) = msg{
                tokio::spawn(self.add_afv(uuid));
            }
        }
    }
    fn uuid(&self) -> BusUuid{
        self.uuid
    }
}

impl Renderable for AfvSelector{
    fn render(&self, ui: &mut eframe::egui::Ui) {
        let mut selected = self.active.blocking_write();
        ComboBox::from_label("Select afv")
        .selected_text(format!("{:x}", *selected))
        .show_ui(ui, |ui|{
            for afv in self.afv.blocking_read().iter(){
                if ui.selectable_value(&mut (*selected), *afv, format!("{:x}",afv)).clicked(){
                    self.handle.spawn(self.bus.clone().send(self.uuid, AfvCtlMessage::Local(LocalMessages::SelectedAfv(*selected))));
                };
            }
        });
    }
}