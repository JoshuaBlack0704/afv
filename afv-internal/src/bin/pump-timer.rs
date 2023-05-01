#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use afv_internal::{w5500::{socket_register::SocketBlock, W5500}, garmin_lidar_v3::GarminLidarV3, lidar::Lidar, stepper::{StepperMotor, StepperOps}, turret::Turret, FLIR_TURRET_PORT, NOZZLE_TURRET_PORT, lights::Lights};
use arduino_hal::{Spi, I2c};
use embedded_hal::spi::{Polarity, Phase};
use panic_halt as _;

// const GATEWAY: [u8;4] = [192,168,4,1];
// const SUBNET: [u8;4] = [255,255,255,0];
// const MAC: [u8;6] = [0x00,0x08,0xdc,0x01,0x02,0x03];
// const IP: [u8;4] = [192,168,4,20];

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);
    // let mut d9 = pins.d9.into_output_high();
    // arduino_hal::delay_ms(3000);
    // d9.set_low();
    // let mut d8 = pins.d8.into_output_high();
    // arduino_hal::delay_ms(3000);
    // d8.set_low();
    // let mut d7 = pins.d7.into_output_high();
    // arduino_hal::delay_ms(3000);
    // d7.set_low();
    let mut serial = arduino_hal::default_serial!(peripherals, pins, 57600);    

    for _ in  0..120{
        arduino_hal::delay_ms(1000);
    }
    let _ = ufmt::uwriteln!(&mut serial, "PUMP ON");
    let mut d7 = pins.d7.into_output_high();
    for _ in  0..300{
        arduino_hal::delay_ms(1000);
    }
    let _ = ufmt::uwriteln!(&mut serial, "PUMP OFF");
    d7.set_low();
    loop{
        
    }
}
