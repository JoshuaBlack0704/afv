#![no_std]
#![no_main]

use afv_internal::servo::Servo;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);

    let d9 = pins.d9;
    let servo_ctl = Servo::new(true, false, peripherals.TC1);
    d9.into_output();

    loop {
    }
}
