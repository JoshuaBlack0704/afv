use serde::{Serialize, Deserialize};
use ufmt::derive::uDebug;

#[derive(uDebug, Serialize, Deserialize, Clone)]
pub enum InternalMessage{
    Ping(u8),
}