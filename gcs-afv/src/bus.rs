use std::sync::Arc;

use async_trait::async_trait;

#[async_trait]
pub trait BusProcess<M>: Send + Sync{
    async fn recieve(self: Arc<Self>, msg: M);
}

#[async_trait]
pub trait BusSender<M>: Send + Sync{
    async fn send(self: Arc<Self>, msg: M);
}

pub struct Bus<M>{
    elements: Arc<dyn BusProcess<M>>,
}

