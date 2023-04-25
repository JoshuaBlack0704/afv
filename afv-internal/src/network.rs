use serde::{Serialize, Deserialize};
use ufmt::derive::uDebug;

use crate::{SOCKET_MSG_SIZE, turret::TurretMsg, lidar::LidarMsg, pump::PumpMsg, lights::LightsMsg, sirens::SirenMsg};

#[derive(uDebug, Serialize, Deserialize, Clone, Copy)]
pub enum InternalMessage{
    Ping(u8),
    Turret(TurretMsg),
    Lidar(LidarMsg),
    Pump(PumpMsg),
    Lights(LightsMsg),
    Siren(SirenMsg),
}

impl InternalMessage{
    pub fn to_msg(&self) -> Option<[u8;SOCKET_MSG_SIZE]>{
        let mut data = [0u8;SOCKET_MSG_SIZE];
        match postcard::to_slice(self, &mut data){
            Ok(_) => return Some(data),
            Err(_) => return None,
        };
        // if let Ok(count) = serde_json_core::to_slice(&self, &mut data){
        //     data[data.len()-1] = count as u8;
        //     return Some(data);
        // }
        // None
    }
    pub fn from_msg(data: &[u8]) -> Option<InternalMessage>{
        match postcard::from_bytes::<Self>(data){
            Ok(msg) => return Some(msg),
            Err(_) => return None,
        }
        // if let Ok((msg, _)) = serde_json_core::from_slice(&data[0..data[data.len()-1] as usize]){
        //     return Some(msg);
        // }
        // None
    }
}