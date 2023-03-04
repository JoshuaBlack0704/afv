use serde::{Serialize, Deserialize};

pub mod bus;

pub const GCSBRIDGEPORT: u16 = 4040;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AfvCtlMessage{
    NetworkAfvUUID(u64),
    NetworkAfvUUIDPoll,
    
}


pub mod networkbus;

pub mod afvbus;

pub mod gcsbus;