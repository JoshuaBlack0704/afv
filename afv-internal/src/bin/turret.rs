#![no_std]
#![no_main]

use afv_internal::servo::Servo;
use arduino_hal::{Spi, spi::Settings};
use panic_halt as _;
use w5500_hl::ll::blocking::fdm::W5500;

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);
    // let cs = pins.d10.into_output();
    // let mosi = pins.d11.into_output();
    // let miso = pins.d12.into_pull_up_input();
    // let sck = pins.d13.into_output();
    // let (spi, _cs) = Spi::new(peripherals.SPI, sck, mosi, miso, cs, Settings::default());

    let servo_ctl = Servo::new(true, true, peripherals.TC1);
    let _d9 = pins.d9.into_output();
    let _d10 = pins.d10.into_output();

    // let w5500 = W5500::new(spi);

    loop {
        servo_ctl.set_pb1_angle(40.0);
        servo_ctl.set_pb2_angle(0.0);
        arduino_hal::delay_ms(1000);
        servo_ctl.set_pb1_angle(-40.0);
        servo_ctl.set_pb2_angle(-90.0);
        arduino_hal::delay_ms(1000);
    }
}
