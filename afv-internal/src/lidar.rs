use serde::{Serialize, Deserialize};
use ufmt::derive::uDebug;

#[derive(uDebug, Serialize, Deserialize, Clone, Copy)]
pub enum LidarMsg{
    PollLidar,
    LidarDistanceCm(u32),
}