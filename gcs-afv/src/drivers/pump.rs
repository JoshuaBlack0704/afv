use afv_internal::{PUMP_PORT, network::InternalMessage, pump::PumpMsg};
use serde::{Serialize, Deserialize};
use tokio::{sync::broadcast, time::{Instant, Duration, timeout, interval}};

use crate::network::{NetMessage, socket::Socket, scanner::{ScanBuilder, ScanCount}};

pub const PUMP_COMMAND_INTERVAL:u64 = 1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PumpDriverMessage{
    TurnOn,
}

#[derive(Clone)]
pub struct PumpDriver{
    net_tx: broadcast::Sender<NetMessage>,
    pump_socket: Socket,
}

impl PumpDriver{
    pub async fn new(net_tx: broadcast::Sender<NetMessage>) -> Option<Self>{
        
        let pump_socket = match ScanBuilder::default().scan_count(ScanCount::Infinite).add_port(PUMP_PORT).dispatch().recv_async().await{
            Ok(stream) => {
                Socket::new(stream, false)
            },
            Err(_) => return None,
        };

        let pump = Self{
            net_tx,
            pump_socket,
        };

        tokio::spawn(pump.clone().forward_messages_task());
        tokio::spawn(pump.clone().command_pump_task());

        Some(pump)
    }

    async fn forward_messages_task(self){
        
    }
    async fn command_pump_task(self){
        let mut net_rx = self.net_tx.subscribe();
        let mut last_cmd = Instant::now();
        let mut interval = interval(Duration::from_secs(PUMP_COMMAND_INTERVAL));

        loop{
            if let Ok(Ok(NetMessage::PumpDriver(PumpDriverMessage::TurnOn))) = timeout(Duration::from_secs(PUMP_COMMAND_INTERVAL + 1), net_rx.recv()).await{
                last_cmd = Instant::now();
            }
            interval.tick().await;

            if Instant::now().duration_since(last_cmd) < Duration::from_secs(PUMP_COMMAND_INTERVAL){
                if let Some(msg) = InternalMessage::Pump(PumpMsg::TurnOn).to_msg(){
                    self.pump_socket.write_data(&msg).await;
                }
                continue;
            }

            if let Some(msg) = InternalMessage::Pump(PumpMsg::TurnOff).to_msg(){
                self.pump_socket.write_data(&msg).await;
            }
        }
    }
}