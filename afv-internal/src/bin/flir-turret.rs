#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use afv_internal::{turret::Turret, lidar::Lidar, stepper::StepperMotor, w5500::{socket_register::SocketBlock, W5500}, FLIR_TURRET_PORT, NOZZLE_TURRET_PORT, garmin_lidar_v3::GarminLidarV3, pump::Pump, lights::Lights, sirens::Siren};
use arduino_hal::{Spi, I2c};
use embedded_hal::spi::{Polarity, Phase};
use panic_halt as _;

const GATEWAY: [u8;4] = [10,192,138,254];
const SUBNET: [u8;4] = [255,255,255,0];
const MAC: [u8;6] = [0x00,0x08,0xdc,0x01,0x02,0x03];
const IP: [u8;4] = [10,192,138,20];

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);
    let mut serial = arduino_hal::default_serial!(peripherals, pins, 57600);    
    let mut cs = pins.d10.into_output();
    cs.set_high();
    let mut mosi = pins.d11.into_output();
    mosi.set_high();
    let miso = pins.d12.into_pull_up_input();
    let sck = pins.d13.into_output();
    let mut settings = arduino_hal::spi::Settings::default();
    settings.clock = arduino_hal::spi::SerialClockRate::OscfOver128;
    settings.mode.polarity = Polarity::IdleLow;
    settings.mode.phase = Phase::CaptureOnFirstTransition; 
    let (mut spi, mut cs) = Spi::new(peripherals.SPI, sck, mosi, miso, cs, settings);
    let mut i2c = I2c::new(peripherals.TWI, pins.a4.into_pull_up_input(), pins.a5.into_pull_up_input(), 1000);
    let (_, _) = W5500::new(Default::default(), GATEWAY, SUBNET, MAC, IP, &mut spi, &mut cs, &mut serial);


    let pan = StepperMotor::new(pins.d5.into_output(), pins.d4.into_output(), None, None, 200, Some(16), 1000, false);
    let tilt = StepperMotor::new(pins.d3.into_output(), pins.d2.into_output(), None, None, 200, Some(16), 1000, true);
    let mut flir_turret = Turret::new(pan, tilt, FLIR_TURRET_PORT, SocketBlock::SOCKET0, &mut spi, &mut cs, &mut serial);
    
    let pan = StepperMotor::new(pins.d9.into_output(), pins.d8.into_output(), None, None, 200, Some(16), 1000, false);
    let tilt = StepperMotor::new(pins.d7.into_output(), pins.d6.into_output(), None, None, 200, Some(16), 1000, true);
    let mut nozzle_turret = Turret::new(pan, tilt, NOZZLE_TURRET_PORT, SocketBlock::SOCKET1, &mut spi, &mut cs, &mut serial);
    
    let mut garmin_lidar = GarminLidarV3::new(None, &mut serial);
    garmin_lidar.start_auto_measurement(&mut i2c, &mut serial);
    let mut lidar = Lidar::new(SocketBlock::SOCKET2, garmin_lidar, &mut spi, &mut cs, &mut serial);

    let mut pump = Pump::new(SocketBlock::SOCKET3, pins.a0.into_output(), &mut spi, &mut cs, &mut serial);
    let mut lights = Lights::new(SocketBlock::SOCKET4, pins.a1.into_output(), &mut spi, &mut cs, &mut serial);
    let mut siren = Siren::new(SocketBlock::SOCKET5, pins.a2.into_output(), &mut spi, &mut cs, &mut serial);



    loop{
        flir_turret.process(&mut spi, &mut cs, &mut serial);
        nozzle_turret.process(&mut spi, &mut cs, &mut serial);
        lidar.process(&mut i2c, &mut spi, &mut cs, &mut serial);
        pump.process(&mut spi, &mut cs, &mut serial);
        lights.process(&mut spi, &mut cs, &mut serial);
        siren.process(&mut spi, &mut cs, &mut serial);
    }
}
