#![no_std]
#![no_main]

use arduino_hal::{spi::SerialClockRate, prelude::_embedded_hal_spi_FullDuplex};
use common_core::bits::Bits;
use embedded_hal::{spi, digital::v2::OutputPin};
use panic_halt as _;
// use w5500::{UninitializedDevice, bus::FourWire, MacAddress, net::Ipv4Addr, Mode};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut v_clock = pins.d2.into_output();
    v_clock.set_low();
    let mut v_data = pins.d3.into_output();
    v_data.set_low();
    let spi = dp.SPI;
    let sclk = pins.d13.into_output();
    let mosi = pins.d11.into_output();
    let miso = pins.d12.into_pull_up_input();
    let mut cs = pins.d10.into_output();
    cs.set_high();
    let mut settings = arduino_hal::spi::Settings::default();
    settings.clock = SerialClockRate::OscfOver128;
    settings.mode = spi::Mode{ polarity: spi::Polarity::IdleLow, phase: spi::Phase::CaptureOnFirstTransition };
    let (mut spi, mut cs) = arduino_hal::spi::Spi::new(spi, sclk, mosi, miso, cs, settings);
    arduino_hal::delay_ms(1000);
    // Set ping in MR on common register
    // let w_addr:u16 = 0x0000;
    // let w_addr = w_addr.to_be_bytes();
    // let mut w_control:[u8;8] = [0,0,0,0,0,1,0,0];
    // let w_control = Bits::from_bits(&mut w_control).byte();
    // let mut write:[u8;8] = [0,0,0,1,0,0,0,0];
    // let write = Bits::from_bits(&mut write).byte();
    // cs.set_low();
    // arduino_hal::delay_ms(1);
    // spi.send(w_addr[0]);
    // arduino_hal::delay_ms(1);
    // spi.send(w_addr[1]);
    // arduino_hal::delay_ms(1);
    // spi.send(w_control);
    // arduino_hal::delay_ms(1);
    // spi.send(write);
    // arduino_hal::delay_ms(1);
    // cs.set_high();
    
    // Get MR from common register
    arduino_hal::delay_ms(1000);
    cs.set_low().unwrap();
    let w_addr:u16 = 0x0000;
    let w_addr = w_addr.to_be_bytes();
    let mut w_control:[u8;8] = [0,0,0,0,0,0,0,0];
    let w_control = Bits::from_bits(&mut w_control).byte();
    let ret:u8 = 0;
    cs.set_low();
    arduino_hal::delay_ms(1);
    spi.send(w_addr[0]);
    arduino_hal::delay_ms(1);
    spi.send(w_addr[1]);
    arduino_hal::delay_ms(1);
    spi.send(w_control);
    arduino_hal::delay_ms(1);
    spi.send(ret);
    arduino_hal::delay_ms(1);
    cs.set_high();
    let ret = spi.read();
    loop {
        if let Ok(v) = ret{
            let byte = Bits::new(&v);
            for b in byte.bits_boolean(){
                v_clock.set_high();
                if *b{
                    v_data.set_high()
                }
                else{
                    v_data.set_low();
                }
                arduino_hal::delay_ms(1);
                v_clock.set_low();
                arduino_hal::delay_ms(1);
            }
            arduino_hal::delay_ms(100);
            
        }
        else{
            v_clock.set_high();
        }
    }
}
