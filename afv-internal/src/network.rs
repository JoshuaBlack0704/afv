use serde::{Deserialize, Serialize};
use ufmt::derive::uDebug;

use crate::{
    lidar::LidarMsg, lights::LightsMsg, pump::PumpMsg, sirens::SirenMsg, turret::TurretMsg,
    SOCKET_MSG_SIZE,
};

#[derive(uDebug, Serialize, Deserialize, Clone, Copy)]
pub enum InternalMessage {
    Ping(u8),
    Turret(TurretMsg),
    Lidar(LidarMsg),
    Pump(PumpMsg),
    Lights(LightsMsg),
    Siren(SirenMsg),
}

impl InternalMessage {
    pub fn to_msg(&self) -> Option<[u8; SOCKET_MSG_SIZE]> {
        let mut data = [0u8; SOCKET_MSG_SIZE];
        match postcard::to_slice(self, &mut data) {
            Ok(_) => return Some(data),
            Err(_) => return None,
        };
    }
    pub fn from_msg(data: &[u8]) -> Option<InternalMessage> {
        match postcard::from_bytes::<Self>(data) {
            Ok(msg) => return Some(msg),
            Err(_) => return None,
        }
    }
}
