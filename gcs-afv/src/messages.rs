
use serde::{Deserialize, Serialize};

use crate::afvbus::AfvUuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetworkMessages{
    PollAfvUuid,
    AfvUuid(AfvUuid),
    FlirStream(AfvUuid),
    #[serde(with = "serde_bytes")]
    NalPacket(Vec<u8>),
    FlirFilterLevel(u8),
    FlirTargetIterations(u32),
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LocalMessages{
    SelectedAfv(AfvUuid),
    FlirStream(AfvUuid),
    #[serde(with = "serde_bytes")]
    NalPacket(Vec<u8>),
    FlirFilterLevel(u32),
    FlirTargetIterations(u32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AfvCtlMessage{
    Network(NetworkMessages),
    Local(LocalMessages),
}
