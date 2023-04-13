use arduino_hal::{clock::MHz16, hal::usart::Usart0};

use crate::stepper::{StepperOps, StepperOpsError};

/// Zero is at direct forward
/// Left is [-max_degrees, 0]
/// Right is [0, max_degrees]
pub struct PanTilt<PS: StepperOps, TS: StepperOps>{
    pan_stepper: PS,
    tilt_stepper: TS,
} 

impl<PS:StepperOps, TS: StepperOps> PanTilt<PS, TS>{
    pub fn new(pan_stepper: PS, tilt_stepper: TS) -> Self {
        Self{
            pan_stepper,
            tilt_stepper,
        }
    }
    /// This algorithim is based off of hard stopping occuring on the turret
    pub fn home(&mut self, serial: &mut Usart0<MHz16>, home_steps: i32) -> Result<(), StepperOpsError>{
        self.pan_stepper.step(home_steps, true, serial)?;
        
        if let Err(_) = self.pan_stepper.to_step(0, serial){
            let _ = ufmt::uwriteln!(serial, "Pan Homing Error");
        }
        
        self.tilt_stepper.step(home_steps, true, serial)?;
        
        if let Err(_) = self.tilt_stepper.to_step(0, serial){
            let _ = ufmt::uwriteln!(serial, "Pan Homing Error");
        }
        

        let _ = ufmt::uwriteln!(serial, "Pan tilt homed");
        Ok(())
    }
}