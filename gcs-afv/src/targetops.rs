use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use rand::{thread_rng, Rng};
use tokio::{sync::RwLock, runtime::Handle, time::{Instant, Duration}};

use crate::{bus::{BusUuid, Bus, BusElement}, afvbus::AfvUuid, messages::{AfvCtlMessage, NetworkMessages, LocalMessages}};

pub const AUTO_TARGET_TIME: Duration = Duration::from_secs(3);
pub struct Local;
pub struct Network;
pub struct TargetComputer<T>{
    bus_uuid: BusUuid,
    afv_uuid: RwLock<AfvUuid>,
    bus: Bus<AfvCtlMessage>,
    handle: Handle,
    _net: PhantomData<T>,
    
    flir_angle: RwLock<Option<(f32, f32)>>,
    distance: RwLock<Option<f32>>,

    nozzle_angle: RwLock<(f32, f32)>,

    auto_target_request: RwLock<Instant>,
}

impl TargetComputer<Local>{
    pub async fn new(bus: Bus<AfvCtlMessage>) -> Arc<Self> {
        let comp = Arc::new(Self{
            bus_uuid: thread_rng().gen(),
            afv_uuid: Default::default(),
            bus: bus.clone(),
            handle: Handle::current(),
            _net: PhantomData,
            flir_angle: Default::default(),
            distance: Default::default(),
            nozzle_angle: Default::default(),
            auto_target_request: RwLock::new(Instant::now()),
        });

        bus.add_element(comp.clone()).await;

        comp
        
    }
    async fn poll_data(self: Arc<Self>){
        *self.flir_angle.write().await = None;
        *self.distance.write().await = None;
        self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Local(LocalMessages::PollFlirAngle(*self.afv_uuid.read().await))).await;
        self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Local(LocalMessages::PollDistance(*self.afv_uuid.read().await))).await;
    }
}

impl TargetComputer<Network>{
    pub async fn new(bus: Bus<AfvCtlMessage>) -> Arc<Self> {
        let comp = Arc::new(Self{
            bus_uuid: thread_rng().gen(),
            afv_uuid: Default::default(),
            bus: bus.clone(),
            handle: Handle::current(),
            _net: PhantomData,
            flir_angle: Default::default(),
            distance: Default::default(),
            nozzle_angle: Default::default(),
            auto_target_request: RwLock::new(Instant::now()),
        });

        bus.add_element(comp.clone()).await;

        comp
        
    }
    async fn poll_data(self: Arc<Self>){
        *self.flir_angle.write().await = None;
        *self.distance.write().await = None;
        self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Network(NetworkMessages::PollFlirAngle(*self.afv_uuid.read().await))).await;
        self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Network(NetworkMessages::PollDistance(*self.afv_uuid.read().await))).await;
    }
}

impl<T> TargetComputer<T>{
    pub async fn set_flir_angle(&self, angle: (f32,f32)){
        *self.flir_angle.write().await = Some(angle);

        self.attempt_solution().await;
    }
    pub async fn set_distance(&self, distance: f32){
        *self.distance.write().await = Some(distance);
        
        self.attempt_solution().await;
    }
    pub async fn attempt_solution(&self){
        let angle = match *self.flir_angle.read().await{
            Some(a) => a,
            None => return,
        };
        let distance = match *self.distance.read().await{
            Some(d) => d,
            None => return,
        };

        // TODO: Develop and send firing solution



        *self.flir_angle.write().await = None;
        *self.distance.write().await = None;
    }
}


/// This variant is spawned on the AFV
#[async_trait]
impl BusElement<AfvCtlMessage> for TargetComputer<Local>{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        if let AfvCtlMessage::Local(msg) = msg{
            match msg{
                LocalMessages::SelectedAfv(uuid) => *self.afv_uuid.write().await = uuid,
                LocalMessages::FlirAngle(_, _, _) => todo!(),
                LocalMessages::Distance(_, _) => todo!(),
                LocalMessages::PollFiringSolution(uuid) => {
                    if !uuid == *self.afv_uuid.read().await{return}
                    tokio::spawn(self.poll_data());
                    
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

/// This variant is spawned on the control station
#[async_trait]
impl BusElement<AfvCtlMessage> for TargetComputer<Network>{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        if let AfvCtlMessage::Local(msg) = msg{
            if let LocalMessages::PollFiringSolution(uuid) = msg{
                if !uuid == *self.afv_uuid.read().await{return}
                tokio::spawn(self.poll_data());
            }
            
            return;
        }
    }
    fn uuid(&self) -> BusUuid{
        self.bus_uuid
    }
}
