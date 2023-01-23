#![no_std]
#![no_main]

use arduino_hal::{spi::SerialClockRate, prelude::_embedded_hal_blocking_spi_Write};
use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let spi = dp.SPI;
    let sclk = pins.d13.into_output();
    let mosi = pins.d11.into_output();
    let miso = pins.d12.into_pull_up_input();
    let mut cs = pins.d10.into_output();
    cs.set_high();
    let mut err = pins.d2.into_output();
    err.set_low();
    let mut settings = arduino_hal::spi::Settings::default();
    settings.clock = SerialClockRate::OscfOver128;
    let (mut spi, mut _cs) = arduino_hal::spi::Spi::new(spi, sclk, mosi, miso, cs, settings);
    let phrase = "Hello";
    let data = phrase.as_bytes();
    loop {
        arduino_hal::delay_ms(1000);
        let _ = _cs.set_low();
        let _ = spi.write(&data);
        let _ = _cs.set_high();
    }
}
