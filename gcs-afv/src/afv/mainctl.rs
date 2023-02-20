use std::sync::Arc;

use afv_internal::{MAINCTLPORT, network::InternalMessage, SOCKET_MSG_SIZE};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::{sync::RwLock, time::{Duration, sleep}, net::{TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}}, io::{AsyncWriteExt, AsyncReadExt}, runtime::Handle};

use crate::{network::{ComEngine, AfvMessage, ComEngineService}, scanner::{Scanner, ScannerHandler}, gui::GuiElement};

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
    Ping,
    PumpState(bool),
}

pub struct MainCtl{
    controller: Arc<dyn Controller>,
    handle: Handle,
}

pub struct Actuator{
    com: Option<Com>,
    stream_reader: RwLock<Option<OwnedReadHalf>>,
    stream_writer: RwLock<Option<OwnedWriteHalf>>,
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
                handle: Handle::current(),
            }
        )
    }
    pub async fn linked(com: Arc<ComEngine<AfvMessage>>) -> Arc<Self>{
       let controller = Link::new(com).await; 
        Arc::new(
            Self{
                controller,
                handle: Handle::current(),
            }
        )
    }
    pub async fn simulated(com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Self>{
       let controller = Simulator::new(com).await; 
        Arc::new(
            Self{
                controller,
                handle: Handle::current(),
            }
        )
    }
}
impl GuiElement for MainCtl{
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        todo!()
    }

    fn name(&self) -> String {
        todo!()
    }

    fn render(self: Arc<Self>, ui: &mut eframe::egui::Ui) {
        if ui.button("Refresh turret").clicked(){
            self.handle.spawn(self.controller.clone().set_pump(true));
        }
    }
}

impl Actuator{
    pub async fn new(com: Option<Arc<ComEngine<AfvMessage>>>) -> Arc<Self>{
        let controller = Arc::new(Self{
            stream_reader: Default::default(),
            stream_writer: Default::default(),
            com: com.clone(),
        });
        tokio::spawn(controller.clone().repeat_connect());
        if let Some(c) = com{
            c.add_listener(controller.clone()).await;
        }
        controller
    }
    async fn repeat_connect(self: Arc<Self>){
        let scanner = Scanner::new(None).await;
        scanner.add_port(MAINCTLPORT).await;
        scanner.set_handler(self.clone()).await;
        while let None = *self.stream_reader.read().await{
            println!("Attempting main control connection");
            scanner.clone().dispatch_all_interfaces();
            sleep(MAINCTL_ATTEMPT_CONNECT_TIME).await;
        }
    }
    async fn reception(self: Arc<Self>){
        loop{
            let mut lock = self.stream_reader.write().await;
            let s = match &mut (*lock){
                Some(s) => s,
                None => {
                    sleep(MAINCTL_ATTEMPT_CONNECT_TIME).await;
                    continue;
                },
            };

            let mut data = [0u8;SOCKET_MSG_SIZE];
            if let Err(_) = s.read_exact(&mut data).await{
                continue;
            }

            if let Some(msg) = InternalMessage::from_msg(data){
                match msg{
                    InternalMessage::Ping(_) => {
                        if let Some(com) = &self.com{
                            let _ = com.send(AfvMessage::MainCtl(MainCtlMsg::Ping)).await;
                            
                        }
                    },
                    InternalMessage::PumpState(_) => {},
                }
            }
        }
        
    }
}
#[async_trait]
impl Controller for Actuator{
    async fn connected(&self) -> bool{
        false
    }
    async fn set_pump(self: Arc<Self>, active: bool){
        if let Some(s) = &mut (*self.stream_writer.write().await){
            let msg = InternalMessage::PumpState(active);
            if let Some(data) = msg.to_msg(){
                println!("MAIN CTL: Sending {} bytes to board for pump state", data.len());
                let _ = s.write(&data).await;
            }
        }
    }
    async fn ping(&self){
        if let Some(s) = &mut (*self.stream_writer.write().await){
            let msg = InternalMessage::Ping(0x01);
            if let Some(data) = msg.to_msg(){
                println!("MAIN CTL: Sending {} bytes to board for ping", data.len());
                let _ =  s.write(&data).await;
            }
        }
    }
}
#[async_trait]
impl ComEngineService<AfvMessage> for Actuator{
    async fn notify(self: Arc<Self>, _com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::MainCtl(m) = msg{
            match m{
                MainCtlMsg::Ping => {
                    self.ping().await;
                },
                MainCtlMsg::PumpState(s) => {
                    self.set_pump(s).await;
                },
            }
        }
    }
}
#[async_trait]
impl ScannerHandler for Actuator{
    async fn handle(self: Arc<Self>, stream: TcpStream){
        println!("MAINCTL ACTUATOR: Connected to ctl at {:?}", stream.peer_addr().expect("Could not get peer addr"));
        let (rd, wr) = stream.into_split();
        *self.stream_reader.write().await = Some(rd);
        *self.stream_writer.write().await = Some(wr);
        tokio::spawn(self.reception());
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
        let _ = self.com.send(AfvMessage::MainCtl(MainCtlMsg::PumpState(active))).await;
    }
    async fn ping(&self){
        let _ = self.com.send(AfvMessage::MainCtl(MainCtlMsg::Ping)).await;
    }
}
#[async_trait]
impl ComEngineService<AfvMessage> for Link{
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage){
        if let AfvMessage::MainCtl(msg) = msg{
            match msg{
                MainCtlMsg::Ping => {
                    println!("MAIN CTL LINK: Received ping from hard ware");
                },
                MainCtlMsg::PumpState(_) => {},
            }
        }
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
