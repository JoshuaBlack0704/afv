use afv_internal::{SIREN_PORT, network::InternalMessage};
use serde::{Serialize, Deserialize};
use tokio::{sync::broadcast, time::{Instant, interval, Duration, timeout}};

use crate::network::{NetMessage, socket::Socket, scanner::{ScanBuilder, ScanCount}};

pub const SIREN_COMMAND_INTERVAL: u64 = 1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum SirenDriverMessage{
    TurnOn,
}

#[derive(Clone)]
pub struct SirenDriver{
    net_tx: broadcast::Sender<NetMessage>,
    light_socket: Socket,
}

impl SirenDriver{
    pub async fn new(net_tx: broadcast::Sender<NetMessage>) -> Option<Self>{
        
        let siren_socket = match ScanBuilder::default().scan_count(ScanCount::Infinite).add_port(SIREN_PORT).dispatch().recv_async().await{
            Ok(stream) => {
                Socket::new(stream, false)
            },
            Err(_) => return None,
        };

        let siren = Self{
            net_tx,
            light_socket: siren_socket,
        };

        tokio::spawn(siren.clone().forward_messages_task());
        tokio::spawn(siren.clone().command_siren_task());

        Some(siren)
    }
    async fn forward_messages_task(self){
        
    }
    async fn command_siren_task(self){
        let mut net_rx = self.net_tx.subscribe();
        let mut last_cmd = Instant::now();
        let mut interval = interval(Duration::from_secs(SIREN_COMMAND_INTERVAL));

        loop{
            if let Ok(Ok(NetMessage::SirenDriver(SirenDriverMessage::TurnOn))) = timeout(Duration::from_secs(SIREN_COMMAND_INTERVAL + 1), net_rx.recv()).await{
                last_cmd = Instant::now();
            }
            interval.tick().await;

            if Instant::now().duration_since(last_cmd) < Duration::from_secs(SIREN_COMMAND_INTERVAL){
                if let Some(msg) = InternalMessage::Siren(afv_internal::sirens::SirenMsg::TurnOn).to_msg(){
                    self.light_socket.write_data(&msg).await;
                }
                continue;
            }

            if let Some(msg) = InternalMessage::Siren(afv_internal::sirens::SirenMsg::TurnOff).to_msg(){
                self.light_socket.write_data(&msg).await;
            }
        }
    }
}