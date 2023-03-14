use std::sync::Arc;

use async_trait::async_trait;
use rand::{thread_rng, Rng};

use crate::{bus::{Bus, BusElement, BusUuid}, messages::{AfvCtlMessage, NetworkMessages}};

use super::AfvUuid;

pub struct PollResponder{
    uuid: BusUuid,
    afv_uuid: AfvUuid,
    bus: Bus<AfvCtlMessage>,
}

impl PollResponder{
    pub async fn new(bus: Bus<AfvCtlMessage>, afv_uuid: AfvUuid){
        let responder = Arc::new(
            Self{
                uuid: thread_rng().gen(),
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
        if let AfvCtlMessage::Network(msg) = msg{
            if let NetworkMessages::PollAfvUuid = msg{
                self.bus.clone().send(self.uuid, AfvCtlMessage::Network(NetworkMessages::AfvUuid(self.afv_uuid))).await;
            }
        }
    }
    fn uuid(&self) -> BusUuid{
        self.uuid
    }
}