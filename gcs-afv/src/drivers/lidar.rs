use afv_internal::{LIDAR_PORT, network::InternalMessage, SOCKET_MSG_SIZE, lidar::LidarMsg};
use log::debug;
use serde::{Serialize, Deserialize};
use tokio::{sync::broadcast, time::{Duration, sleep}};

use crate::network::{NetMessage, socket::Socket, scanner::{ScanCount, ScanBuilder}};

pub const POLL_LIDAR_INTERNVAL: u64 = 1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum LidarDriverMessage{
    PollLidar,
    LidarDistanceCm(u32),
}

#[derive(Clone)]
pub struct LidarDriver{
    net_tx: broadcast::Sender<NetMessage>,
    lidar_socket: Socket,
}

impl LidarDriver{
    pub async fn new(net_tx: broadcast::Sender<NetMessage>) -> Option<Self>{
        let lidar_socket = match ScanBuilder::default().scan_count(ScanCount::Infinite).add_port(LIDAR_PORT).dispatch().recv_async().await{
            Ok(stream) => {
                Socket::new(stream, false)
            },
            Err(_) => return None,
        };

        let lidar = Self{
            net_tx,
            lidar_socket,
        };

        tokio::spawn(lidar.clone().forward_messages_task());
        tokio::spawn(lidar.clone().poll_lidar_task());

        Some(lidar)
    }

    async fn forward_messages_task(self){
        loop{
            let mut data = [0u8; SOCKET_MSG_SIZE];
            for i in 0..data.len(){
                data[i] = self.lidar_socket.read_byte().await;
            }

            match InternalMessage::from_msg(&data){
                Some(InternalMessage::Lidar(LidarMsg::LidarDistanceCm(distance))) => {
                    let _ = self.net_tx.send(NetMessage::LidarDriver(LidarDriverMessage::LidarDistanceCm(distance)));
                    println!("Lidar Distance: {:?} cm", distance);
                }
                _ => {}
            }
        }
    }
    async fn poll_lidar_task(self){
        loop{
            sleep(Duration::from_secs(POLL_LIDAR_INTERNVAL)).await;
            debug!("Polling lidar for distance");
            if let Some(msg) = InternalMessage::Lidar(LidarMsg::PollLidar).to_msg(){
                self.lidar_socket.write_data(&msg).await;
            }
        }
    }
}