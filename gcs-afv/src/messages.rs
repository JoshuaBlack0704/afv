use serde::{Deserialize, Serialize};

use crate::afvbus::AfvUuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetworkMessages{
    PollAfvUuid,
    AfvUuid(AfvUuid),
    FlirStream(AfvUuid),
    NalPacket(Vec<u8>)
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LocalMessages{
    SelectedAfv(AfvUuid),
    FlirStream(AfvUuid),
    NalPacket(Vec<u8>)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AfvCtlMessage{
    Network(NetworkMessages),
    Local(LocalMessages),
}
