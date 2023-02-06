use std::sync::Arc;

use tokio::net::ToSocketAddrs;

use crate::network::{ComEngine, AfvMessage};

pub mod flir;

#[allow(unused)]
pub struct Afv{
    com: Arc<ComEngine<AfvMessage>>,
    
}

impl Afv{
    pub fn new(com: Arc<ComEngine<AfvMessage>>) -> Arc<Afv> {
        Arc::new(
            Self{
                com,
            }
        )
    }
    pub fn link(com: Arc<ComEngine<AfvMessage>>) -> Arc<Afv>{
        Arc::new(
            Self{
                com,
            }
        )
    }

    pub async fn dummy(addr: impl ToSocketAddrs) -> Arc<Afv> {
        
        let com = ComEngine::afv_com_listen(addr).await.expect("Dummy afv could not establish connection");
        Arc::new(Self{
            com,
        })
    }
    
}

