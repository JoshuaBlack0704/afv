use std::{marker::PhantomData, sync::Arc};

pub struct NozzleTurret<NetType>{
    _net: PhantomData<NetType>,
}

impl<T> NozzleTurret<T>{
    pub async fn new() -> Arc<Self> {
        Arc::new(Self{
            _net: PhantomData,
        })
    }
    
}