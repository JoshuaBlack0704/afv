use serde::{Serialize, Deserialize};

use crate::{drivers::{flir::FlirDriverMessage, turret::TurretDriverMessage, lidar::LidarDriverMessage, pump::PumpDriverMessage, siren::SirenDriverMessage, lights::LightsDriverMessage}, operators::{flir::FlirOperatorMessage, nozzle::NozzleOperatorMessage, pump::PumpOperatorMessage}};

#[derive(Serialize, Deserialize, Clone)]
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
    PeripheralOperator(PumpOperatorMessage),
}

pub mod scanner;

pub mod socket;

pub mod afv_bridge;

pub mod gcs_bridge;