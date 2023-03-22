use std::{marker::PhantomData, sync::Arc};

pub struct DistanceSensor<NetType>{
    _net: PhantomData<NetType>,
}

impl<T> DistanceSensor<T>{
    pub async fn new() -> Arc<Self> {
        Arc::new(Self{
            _net: PhantomData,
        })
    }
    
}