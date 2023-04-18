use tokio::sync::broadcast;

use crate::network::NetMessage;

pub trait Renderable{
    
}

struct GcsArgs{
    
}

pub struct GcsUi{
    
}

impl GcsUi{
    pub fn launch(){
        let (tx, rx) = broadcast::channel::<NetMessage>(10000);
        
        
    }
}