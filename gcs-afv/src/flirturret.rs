use std::{marker::PhantomData, sync::Arc};

pub struct FlirTurret<NetType>{
    _net: PhantomData<NetType>,
}

impl<T> FlirTurret<T>{
    pub async fn new() -> Arc<Self> {
        Arc::new(Self{
            _net: PhantomData,
        })
    }

    pub async fn adjust_angle(&self, angles: (f32, f32)){
        
    }
    
}