use std::sync::Arc;

use async_trait::async_trait;
use rand::{thread_rng, Rng};

use crate::{bus::{Bus, BusElement}, AfvCtlMessage};

pub struct PollResponder{
    uuid: u64,
    afv_uuid: u64,
    bus: Bus<AfvCtlMessage>,
}

impl PollResponder{
    pub async fn new(bus: Bus<AfvCtlMessage>, afv_uuid: u64){
        let responder = Arc::new(
            Self{
                uuid: thread_rng().gen::<u64>(),
                afv_uuid,
                bus: bus.clone(),
            }
        );

        bus.add_element(responder.clone()).await;
    }
}

#[async_trait]
impl BusElement<AfvCtlMessage> for PollResponder{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        match msg{
            AfvCtlMessage::NetworkAfvUUIDPoll => {
                self.bus.send(self.uuid, AfvCtlMessage::NetworkAfvUUID(self.afv_uuid)).await;
            },
            _ => {}
        }
    }
    fn uuid(&self) -> u64{
        self.uuid
    }
}