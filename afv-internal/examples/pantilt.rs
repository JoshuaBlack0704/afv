#![no_std]
#![no_main]

use afv_internal::{stepper::StepperMotor, pantilt::PanTilt};
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);
    let mut serial = arduino_hal::default_serial!(peripherals, pins, 57600);    
    let _ = ufmt::uwriteln!(&mut serial, "Starting Pan Tilt");

    let pan = StepperMotor::new(pins.d5.into_output(), pins.d4.into_output(), None, None, 200, Some(16), 1000, false);
    let tilt = StepperMotor::new(pins.d3.into_output(), pins.d2.into_output(), None, None, 200, Some(16), 1000, true);

    let mut turret = PanTilt::new(pan, tilt);

    let _ = turret.home(&mut serial, 200);
    
    loop{
        
    }
}

