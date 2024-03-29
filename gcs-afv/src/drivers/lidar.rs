use afv_internal::{lidar::LidarMsg, network::InternalMessage, LIDAR_PORT, SOCKET_MSG_SIZE};
use log::debug;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::broadcast,
    time::{sleep, Duration},
};

use crate::network::{
    scanner::{ScanBuilder, ScanCount},
    socket::Socket,
    NetMessage,
};

pub const POLL_LIDAR_INTERNVAL: Duration = Duration::from_millis(500);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum LidarDriverMessage {
    PollLidar,
    LidarDistanceCm(u32),
}

#[derive(Clone)]
/// The LidarDriver struct manages the connection the the AFV's Lidar. It does this by repeatedly sending distance readings on the main bus
///
/// Port addressing in this sense means that each "Turret" that is run on an Arduino starts its own TCP server
/// on a specific port. This means that no matter what IP address/Arduino a specific turret is run on it can still be 
/// found automatically.
pub struct LidarDriver {
    net_tx: broadcast::Sender<NetMessage>,
    lidar_socket: Socket,
}

impl LidarDriver {
    /// This function auto connects to the Lidar firmware running on an Arduino and then spawns the monitoring tasks on the bus.
    pub async fn new(net_tx: broadcast::Sender<NetMessage>) -> Option<Self> {
        let lidar_socket = match ScanBuilder::default()
            .scan_count(ScanCount::Infinite)
            .add_port(LIDAR_PORT)
            .dispatch()
            .recv_async()
            .await
        {
            Ok(stream) => Socket::new(stream, false),
            Err(_) => return None,
        };

        let lidar = Self {
            net_tx,
            lidar_socket,
        };

        tokio::spawn(lidar.clone().forward_messages_task());
        tokio::spawn(lidar.clone().poll_lidar_task());

        Some(lidar)
    }

    /// This task is responsible for forwarding tasks sent from the host target lidar process to the main bus
    async fn forward_messages_task(self) {
        loop {
            let mut data = [0u8; SOCKET_MSG_SIZE];
            for i in 0..data.len() {
                data[i] = self.lidar_socket.read_byte().await;
            }

            match InternalMessage::from_msg(&data) {
                Some(InternalMessage::Lidar(LidarMsg::LidarDistanceCm(distance))) => {
                    let _ = self.net_tx.send(NetMessage::LidarDriver(
                        LidarDriverMessage::LidarDistanceCm(distance),
                    ));
                    println!("Lidar Distance: {:?} cm", distance);
                }
                _ => {}
            }
        }
    }
    /// This task is responsible for polling distance from the host target lidar process
    async fn poll_lidar_task(self) {
        loop {
            sleep(POLL_LIDAR_INTERNVAL).await;
            debug!("Polling lidar for distance");
            if let Some(msg) = InternalMessage::Lidar(LidarMsg::PollLidar).to_msg() {
                self.lidar_socket.write_data(&msg).await;
            }
        }
    }
}
