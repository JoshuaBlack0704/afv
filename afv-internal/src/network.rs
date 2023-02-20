use serde::{Serialize, Deserialize};
use ufmt::derive::uDebug;

#[derive(uDebug, Serialize, Deserialize, Clone, Copy)]
pub enum InternalMessage{
    Ping(u8),
    ActivatePump,
}