
use serde::{Deserialize, Serialize};

use crate::afvbus::AfvUuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetworkMessages{
    PollAfvUuid,
    AfvUuid(AfvUuid),
    FlirStream(AfvUuid),
    #[serde(with = "serde_bytes")]
    NalPacket(Vec<u8>),
    FlirFilterLevel(AfvUuid, u8),
    FlirTargetIterations(AfvUuid, u32),
    PollFlirAngle(AfvUuid),
    PollDistance(AfvUuid),
    FlirAngle(AfvUuid, f32, f32),
    Distance(AfvUuid, f32),
    PollFiringSolution(AfvUuid),
    /// This is only sent from the ground station
    AutoTarget(AfvUuid),
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LocalMessages{
    SelectedAfv(AfvUuid),
    FlirStream(AfvUuid),
    #[serde(with = "serde_bytes")]
    NalPacket(Vec<u8>),
    FlirFilterLevel(AfvUuid, u8),
    FlirTargetIterations(AfvUuid, u32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AfvCtlMessage{
    Network(NetworkMessages),
    Local(LocalMessages),
}
