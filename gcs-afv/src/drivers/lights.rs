use afv_internal::{LIGHTS_PORT, network::InternalMessage, lights::LightsMsg};
use serde::{Serialize, Deserialize};
use tokio::{sync::broadcast, time::{Instant, interval, Duration, timeout}};

use crate::network::{NetMessage, socket::Socket, scanner::{ScanBuilder, ScanCount}};

pub const LIGHTS_COMMAND_INTERVAL:u64 = 1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum LightsDriverMessage{
    TurnOn,
}

#[derive(Clone)]
pub struct LightsDriver{
    net_tx: broadcast::Sender<NetMessage>,
    light_socket: Socket,
}

impl LightsDriver{
    pub async fn new(net_tx: broadcast::Sender<NetMessage>) -> Option<Self>{
        
        let light_socket = match ScanBuilder::default().scan_count(ScanCount::Infinite).add_port(LIGHTS_PORT).dispatch().recv_async().await{
            Ok(stream) => {
                Socket::new(stream, false)
            },
            Err(_) => return None,
        };

        let lights = Self{
            net_tx,
            light_socket,
        };

        tokio::spawn(lights.clone().forward_messages_task());
        tokio::spawn(lights.clone().command_lights_task());

        Some(lights)
    }
    async fn forward_messages_task(self){
        
    }
    async fn command_lights_task(self){
        let mut net_rx = self.net_tx.subscribe();
        let mut last_cmd = Instant::now();
        let mut interval = interval(Duration::from_secs(LIGHTS_COMMAND_INTERVAL));

        loop{
            if let Ok(Ok(NetMessage::LightDriver(LightsDriverMessage::TurnOn))) = timeout(Duration::from_secs(LIGHTS_COMMAND_INTERVAL + 1), net_rx.recv()).await{
                last_cmd = Instant::now();
            }
            interval.tick().await;

            if Instant::now().duration_since(last_cmd) < Duration::from_secs(LIGHTS_COMMAND_INTERVAL){
                if let Some(msg) = InternalMessage::Lights(LightsMsg::TurnOn).to_msg(){
                    self.light_socket.write_data(&msg).await;
                }
                continue;
            }

            if let Some(msg) = InternalMessage::Lights(LightsMsg::TurnOff).to_msg(){
                self.light_socket.write_data(&msg).await;
            }
        }
    }
}