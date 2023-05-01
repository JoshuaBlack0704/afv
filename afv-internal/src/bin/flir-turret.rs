#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use afv_internal::{w5500::{socket_register::SocketBlock, W5500}, garmin_lidar_v3::GarminLidarV3, lidar::Lidar, stepper::{StepperMotor, StepperOps}, turret::Turret, FLIR_TURRET_PORT, NOZZLE_TURRET_PORT, lights::Lights};
use arduino_hal::{Spi, I2c};
use embedded_hal::spi::{Polarity, Phase};
use panic_halt as _;

const GATEWAY: [u8;4] = [192,168,4,1];
const SUBNET: [u8;4] = [255,255,255,0];
const MAC: [u8;6] = [0x00,0x08,0xdc,0x01,0x02,0x03];
const IP: [u8;4] = [192,168,4,20];

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


    let mut pan = StepperMotor::new(pins.d3.into_output(), pins.d2.into_output(), 92, -800, Some(16), 1000, 500, true);
    pan.home(250, &mut serial);
    let mut tilt = StepperMotor::new(pins.d5.into_output(), pins.d4.into_output(), 266, -60, Some(16), 2000, 1000, false);
    tilt.home(266, &mut serial);
    let mut flir_turret = Turret::new(pan, tilt, FLIR_TURRET_PORT, SocketBlock::SOCKET0, &mut spi, &mut cs, &mut serial);
    
    let mut pan = StepperMotor::new(pins.a0.into_output(), pins.a1.into_output(), 330, -1000, Some(16), 1000, 500, false);
    pan.home(300, &mut serial);
    let mut tilt = StepperMotor::new(pins.a2.into_output(), pins.a3.into_output(), 50, -60, Some(16), 2000, 1000, true);
    tilt.home(-30, &mut serial);
    let mut nozzle_turret = Turret::new(pan, tilt, NOZZLE_TURRET_PORT, SocketBlock::SOCKET1, &mut spi, &mut cs, &mut serial);
    
    let mut garmin_lidar = GarminLidarV3::new(None, &mut serial);
    garmin_lidar.start_auto_measurement(&mut i2c, &mut serial);
    let mut lidar = Lidar::new(SocketBlock::SOCKET2, garmin_lidar, &mut spi, &mut cs, &mut serial);

    // let mut pump = Pump::new(SocketBlock::SOCKET3, pins.a0.into_output(), &mut spi, &mut cs, &mut serial);
    let mut lights = Lights::new(SocketBlock::SOCKET4, pins.d8.into_output(), &mut spi, &mut cs, &mut serial);
    // let mut siren = Siren::new(SocketBlock::SOCKET5, pins.a2.into_output(), &mut spi, &mut cs, &mut serial);



    let _ = ufmt::uwriteln!(&mut serial, "Staring Flir turret loop");
    loop{
        // lidar.poll_distance(&mut i2c, &mut spi, &mut cs, &mut serial);
        flir_turret.process(&mut spi, &mut cs, &mut serial);
        nozzle_turret.process(&mut spi, &mut cs, &mut serial);
        lidar.process(&mut i2c, &mut spi, &mut cs, &mut serial);
        // pump.process(&mut spi, &mut cs, &mut serial);
        lights.process(&mut spi, &mut cs, &mut serial);
        // siren.process(&mut spi, &mut cs, &mut serial);
    }
}
