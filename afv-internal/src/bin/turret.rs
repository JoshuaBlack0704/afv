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
    let cs = pins.d10.into_output();
    let mosi = pins.d11.into_output();
    let miso = pins.d12.into_pull_up_input();
    let sck = pins.d13.into_output();
    let (spi, _cs) = Spi::new(peripherals.SPI, sck, mosi, miso, cs, Settings::default());

    let d9 = pins.d9;
    let servo_ctl = Servo::new(true, false, peripherals.TC1);
    d9.into_output();

    let w5500 = W5500::new(spi);

    loop {
    }
}
