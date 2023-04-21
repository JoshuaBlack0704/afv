use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TurretDriverMessage{
    SetAngle(u16, [f32; 2]),
}

pub struct TurretDriver{
    
}
