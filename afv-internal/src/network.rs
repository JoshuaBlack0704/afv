use serde::{Serialize, Deserialize};
use ufmt::derive::uDebug;

use crate::{SOCKET_MSG_SIZE, turret::TurretMsg, lidar::LidarMsg};

#[derive(uDebug, Serialize, Deserialize, Clone, Copy)]
pub enum InternalMessage{
    Ping(u8),
    FlirSignatureOffset([u8;2]),
    Turret(TurretMsg),
    Lidar(LidarMsg),
}

impl InternalMessage{
    pub fn to_msg(&self) -> Option<[u8;SOCKET_MSG_SIZE]>{
        let mut data = [0u8;SOCKET_MSG_SIZE];
        if let Ok(count) = serde_json_core::to_slice(&self, &mut data){
            data[data.len()-1] = count as u8;
            return Some(data);
        }
        None
    }
    pub fn from_msg(data: &[u8]) -> Option<InternalMessage>{
        if let Ok((msg, _)) = serde_json_core::from_slice(&data[0..data[data.len()-1] as usize]){
            return Some(msg);
        }
        None
    }
}