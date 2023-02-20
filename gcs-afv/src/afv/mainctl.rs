use std::{sync::Arc, net::SocketAddr};

use afv_internal::{TESTPORT, network::InternalMessage, SOCKET_MSG_SIZE};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::{sync::RwLock, time::{Duration, sleep}, net::TcpStream, io::AsyncWriteExt};

use crate::{network::{ComEngine, AfvMessage, ComEngineService}, scanner::{Scanner, ScannerHandler}};

#[async_trait]
pub trait Controller: Send + Sync{
    async fn connected(&self) -> bool;
    async fn set_pump(self: Arc<Self>, active: bool);
    async fn ping(&self);
}

pub const MAINCTL_ATTEMPT_CONNECT_TIME: Duration = Duration::from_secs(1);
type Com = Arc<ComEngine<AfvMessage>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MainCtlMsg{
    Ping
}

pub struct MainCtl{
    controller: Arc<dyn Controller>,
}

pub struct Actuator{
    stream: RwLock<Option<TcpStream>>,
}

pub struct Link{
    com: Com,
}

pub struct Simulator{}

impl MainCtl{
    pub async fn actuated(com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Self>{
       let controller = Actuator::new(com).await; 
        Arc::new(
            Self{
                controller,
            }
        )
    }
    pub async fn linked(com: Arc<ComEngine<AfvMessage>>) -> Arc<Self>{
       let controller = Link::new(com).await; 
        Arc::new(
            Self{
                controller,
            }
        )
    }
    pub async fn simulated(com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Self>{
       let controller = Simulator::new(com).await; 
        Arc::new(
            Self{
                controller,
            }
        )
    }
}

impl Actuator{
    pub async fn new(com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Self>{
        let controller = Arc::new(Self{
            stream: Default::default(),
        });
        tokio::spawn(controller.clone().repeat_connect());
        if let Some(c) = com{
            c.add_listener(controller.clone()).await;
        }
        controller
    }
    async fn repeat_connect(self: Arc<Self>){
        let scanner = Scanner::new(None).await;
        scanner.add_port(TESTPORT).await;
        scanner.set_handler(self.clone()).await;
        while let None = *self.stream.read().await{
            println!("Attempting main control connection");
            scanner.clone().dispatch_all_interfaces();
            sleep(MAINCTL_ATTEMPT_CONNECT_TIME).await;
        }
    }
}
#[async_trait]
impl Controller for Actuator{
    async fn connected(&self) -> bool{
        false
    }
    async fn set_pump(self: Arc<Self>, active: bool){
    }
    async fn ping(&self){
        
    }
}
#[async_trait]
impl ComEngineService<AfvMessage> for Actuator{
    async fn notify(self: Arc<Self>, _com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::MainCtl(m) = msg{
            match m{
                MainCtlMsg::Ping => {
                    if let Some(s) = &mut (*self.stream.write().await){
                        let msg = InternalMessage::Ping(0x01);
                        if let Ok(msg) = serde_json_core::to_vec::<InternalMessage, SOCKET_MSG_SIZE>(&msg){
                            println!("Sending {} bytes to board!", msg.len());
                            let _ = s.write(&msg).await;
                        }
                    }
                },
            }
        }
    }
}
#[async_trait]
impl ScannerHandler for Actuator{
    async fn handle(self: Arc<Self>, stream: TcpStream){
        println!("MAINCTL ACTUATOR: Connected to ctl at {:?}", stream.peer_addr().expect("Could not get peer addr"));
        *self.stream.write().await = Some(stream);
    }
}

impl Link{
    pub async fn new(com: Arc<ComEngine<AfvMessage>>) -> Arc<Self>{
        let link = Arc::new(
            Self { com: com.clone()  }
        );

        com.add_listener(link.clone()).await;
        tokio::spawn(link.clone().interval_ping());
        link
    }
    async fn interval_ping(self: Arc<Self>){
        loop{
            sleep(Duration::from_secs(5)).await;
            self.clone().ping().await;
        }
    }
}
#[async_trait]
impl Controller for Link{
    async fn connected(&self) -> bool{
        false
    }
    async fn set_pump(self: Arc<Self>, active: bool){
    }
    async fn ping(&self){
        let _ = self.com.send(AfvMessage::MainCtl(MainCtlMsg::Ping)).await;
    }
}
#[async_trait]
impl ComEngineService<AfvMessage> for Link{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
    }
}

impl Simulator{
    pub async fn new(com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Self>{
        Arc::new(Self{
            
        })
    }
}
#[async_trait]
impl Controller for Simulator{
    async fn connected(&self) -> bool{
        false
    }
    async fn set_pump(self: Arc<Self>, active: bool){
    }
    async fn ping(&self){
    }
}
#[async_trait]
impl ComEngineService<AfvMessage> for Simulator{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
    }
}
