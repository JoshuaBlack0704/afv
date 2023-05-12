use serde::{Deserialize, Serialize};

use crate::{
    drivers::{
        flir::FlirDriverMessage, lidar::LidarDriverMessage, lights::LightsDriverMessage,
        pump::PumpDriverMessage, siren::SirenDriverMessage, turret::TurretDriverMessage,
    },
    operators::{
        flir::FlirOperatorMessage, naming::NamingOperatorMessage, nozzle::NozzleOperatorMessage,
        peripheral::PeripheralMessage, pump::PumpOperatorMessage,
    },
};

pub const AFV_COMM_PORT: u16 = 4040;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
/// These are the core messages that processes on the bus use to communicate
/// They are split into the logical sections/systems that comprise the code base
pub enum NetMessage {
    FlirDriver(FlirDriverMessage),
    TurretDriver(TurretDriverMessage),
    LidarDriver(LidarDriverMessage),
    PumpDriver(PumpDriverMessage),
    SirenDriver(SirenDriverMessage),
    LightDriver(LightsDriverMessage),
    FlirOperator(FlirOperatorMessage),
    NozzleOperator(NozzleOperatorMessage),
    PumpOperator(PumpOperatorMessage),
    PeripheralOperator(PeripheralMessage),
    NamingOperator(NamingOperatorMessage),
}

/// This module contains the Scanner struct that is resposible for finding all network entites without manual supervision
pub mod scanner;

/// This module contains a Socket wrapper that automatically reconnects if connections are lost
pub mod socket;

/// This module contains the AfvBridge sysytem that enable transparent merging of multiple networks using TCP
pub mod afv_bridge;
