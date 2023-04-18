use tokio::sync::broadcast;

use crate::network::{NetMessage, afv_bridge::AfvBridge, scanner::ScanCount};

pub trait Renderable{
    
}

struct GcsArgs{
    
}

pub struct GcsUi{
    
}

impl GcsUi{
    pub fn launch(){
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not start tokio runtime");
        let (tx, rx) = broadcast::channel::<NetMessage>(10000);
        runtime.spawn(AfvBridge::server(tx.clone(), tx.clone(), None));
        let (tx, rx) = broadcast::channel::<NetMessage>(10000);
        runtime.block_on(AfvBridge::client(tx.clone(), tx.clone(), Default::default()));
        
    }
}