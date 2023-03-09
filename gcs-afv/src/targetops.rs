use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use eframe::egui::Ui;
use rand::{thread_rng, Rng};
use tokio::time::{sleep, Duration, Instant};
use tokio::{runtime::Handle, sync::RwLock};

use crate::bus::BusElement;
use crate::messages::{NetworkMessages, LocalMessages};
use crate::networkbus::Local;
use crate::nozzleturret::NozzleTurret;
use crate::{
    afvbus::AfvUuid,
    bus::{Bus, BusUuid},
    distancesensor::DistanceSensor,
    flirops::FlirController,
    flirturret::FlirTurret,
    messages::AfvCtlMessage,
    networkbus::Network,
};

const AUTOTARGETWAITTIME: Duration = Duration::from_secs(3);
const AUTOTARGETREQUESTTIME: Duration = Duration::from_secs(3);
pub struct TargetingComputer<NetType> {
    bus_uuid: BusUuid,
    afv_uuid: RwLock<AfvUuid>,
    bus: Bus<AfvCtlMessage>,
    handle: Handle,

    flir: Arc<FlirController<NetType>>,
    flir_turret: Arc<FlirTurret<NetType>>,
    nozzle_turret: Arc<NozzleTurret<NetType>>,
    distance_sensor: Arc<DistanceSensor<NetType>>,
    _net: PhantomData<NetType>,

    auto_target: RwLock<bool>,
    auto_target_request: RwLock<Instant>,
}

impl TargetingComputer<Network> {
    pub async fn new(
        bus: Bus<AfvCtlMessage>,
        flir: Arc<FlirController<Network>>,
        flir_turret: Arc<FlirTurret<Network>>,
        nozzle_turret: Arc<NozzleTurret<Network>>,
        distance_sensor: Arc<DistanceSensor<Network>>,
    ) -> Arc<Self> {
        let comp = Arc::new(Self {
            bus_uuid: thread_rng().gen(),
            afv_uuid: Default::default(),
            bus: bus.clone(),
            handle: Handle::current(),
            flir,
            flir_turret,
            nozzle_turret,
            distance_sensor,
            _net: PhantomData,
            auto_target: Default::default(),
            auto_target_request: RwLock::new(Instant::now()),
        });

        tokio::spawn(comp.clone().auto_target_task());

        comp
    }

    pub fn auto_target_state(&self) -> bool {
        *self.auto_target.blocking_read()
    }
    pub fn auto_target_button(&self, ui: &mut Ui) {
        let mut auto_target = self.auto_target.blocking_write();
        if ui
            .selectable_label(*auto_target, "Auto Targeting")
            .clicked()
        {
            *auto_target = !*auto_target;
        }
    }

    async fn auto_target_task(self: Arc<Self>) {
        loop {
            sleep(AUTOTARGETWAITTIME).await;
            if !*self.auto_target.read().await {
                continue;
            }

            // Auto targeting must be enabled

            self.bus
                .clone()
                .send(
                    self.bus_uuid,
                    AfvCtlMessage::Network(NetworkMessages::AutoTarget(
                        *self.afv_uuid.read().await,
                    )),
                )
                .await;
        }
    }
}

impl TargetingComputer<Local>{
    pub async fn new(
        bus: Bus<AfvCtlMessage>,
        flir: Arc<FlirController<Local>>,
        flir_turret: Arc<FlirTurret<Local>>,
        nozzle_turret: Arc<NozzleTurret<Local>>,
        distance_sensor: Arc<DistanceSensor<Local>>,
    ) -> Arc<Self> {
        let comp = Arc::new(Self {
            bus_uuid: thread_rng().gen(),
            afv_uuid: Default::default(),
            bus: bus.clone(),
            handle: Handle::current(),
            flir,
            flir_turret,
            nozzle_turret,
            distance_sensor,
            _net: PhantomData,
            auto_target: Default::default(),
            auto_target_request: RwLock::new(Instant::now()),
        });

        tokio::spawn(comp.clone().auto_target_task());

        comp
    }
    async fn auto_target_task(self: Arc<Self>) {
        loop {
            sleep(AUTOTARGETWAITTIME).await;
            if !*self.auto_target.read().await {
                continue;
            }

            // Auto targeting must be enabled

            self.bus
                .clone()
                .send(
                    self.bus_uuid,
                    AfvCtlMessage::Network(NetworkMessages::AutoTarget(
                        *self.afv_uuid.read().await,
                    )),
                )
                .await;
        }
    }

}

#[async_trait]
impl<T: Send + Sync + 'static> BusElement<AfvCtlMessage> for TargetingComputer<T>{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        match msg{
            AfvCtlMessage::Network(msg) => {
                match msg{
                    NetworkMessages::AutoTarget(afv_uuid) => {
                        if afv_uuid != *self.afv_uuid.read().await{return}

                        tokio::spawn(async move{
                            if let Some(i) = Instant::now().checked_add(AUTOTARGETREQUESTTIME){
                                *self.auto_target_request.write().await = i;
                            }
                        });
                        
                    },
                    _ => {}
                }
            },
            AfvCtlMessage::Local(msg) => {
                match msg{
                    LocalMessages::SelectedAfv(afv_uuid) => {
                        *self.afv_uuid.write().await = afv_uuid;
                    },
                    _ => {}
                }
            },
        }

        
    }
    fn uuid(&self) -> BusUuid{
        self.bus_uuid
    }
}
