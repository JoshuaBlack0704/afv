use serde::{Serialize, Deserialize};

use crate::{drivers::{flir::FlirDriverMessage, turret::TurretDriverMessage, lidar::LidarDriverMessage, pump::PumpDriverMessage, siren::SirenDriverMessage, lights::LightsDriverMessage}, operators::{flir::FlirOperatorMessage, nozzle::NozzleOperatorMessage, pump::PumpOperatorMessage, naming::NamingOperatorMessage, peripheral::PeripheralMessage}};

pub const AFV_COMM_PORT: u16 = 4040;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
/// These are the core messages that processes on the bus use to communicate
/// They are split into the logical sections/systems that comprise the code base
pub enum NetMessage{
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

pub mod scanner;

pub mod socket;

pub mod afv_bridge;
