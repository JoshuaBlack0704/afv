use std::sync::Arc;

use async_trait::async_trait;
use tokio::{sync::RwLock, runtime::{Handle, Runtime}};

pub type BusUuid = u64;

#[async_trait]
pub trait BusElement<M>: Send + Sync{
    async fn recieve(self: Arc<Self>, msg: M);
    fn uuid(&self) -> BusUuid;
}

#[derive(Clone)]
pub struct Bus<M>{
    handle: Handle,
    elements: Arc<RwLock<Vec<Arc<dyn BusElement<M>>>>>,
}

impl<M: Clone> Bus <M>{
    pub async fn new() -> Bus<M> {
        Self{
            elements: Default::default(),
            handle: Handle::current(),
        }
    }
    pub fn new_blocking(rt: &Arc<Runtime>) -> Bus<M> {
        rt.block_on(Self::new())
    }
    pub async fn send(self, sender_id: BusUuid, msg: M){
        let elements = self.elements.read().await;
        for e in elements.iter(){
            if e.uuid() != sender_id{
                e.clone().recieve(msg.clone()).await;
            }
        }
    }
    pub fn send_blocking(&self, sender_id: BusUuid, msg: M){
        self.handle.block_on(self.clone().send(sender_id, msg));
    }
    pub async fn add_element(&self, element: Arc<dyn BusElement<M>>){
        let mut elements = self.elements.write().await;
        for e in elements.iter(){
            if e.uuid() == element.uuid(){
                return;
            }
        }
        elements.push(element);
    }
    pub fn add_element_blocking(&self, element: Arc<dyn BusElement<M>>){
        self.handle.block_on(self.add_element(element));
    }
}

