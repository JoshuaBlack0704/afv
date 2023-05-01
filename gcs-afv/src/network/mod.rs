use serde::{Serialize, Deserialize};

use crate::{drivers::{flir::FlirDriverMessage, turret::TurretDriverMessage, lidar::LidarDriverMessage, pump::PumpDriverMessage, siren::SirenDriverMessage, lights::LightsDriverMessage}, operators::{flir::FlirOperatorMessage, nozzle::NozzleOperatorMessage, pump::PumpOperatorMessage, naming::NamingOperatorMessage, peripheral::PeripheralMessage}};

pub const AFV_COMM_PORT: u16 = 4040;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

pub mod gcs_bridge;