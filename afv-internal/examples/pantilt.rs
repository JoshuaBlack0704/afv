#![no_std]
#![no_main]

use afv_internal::{stepper::StepperMotor, turret::Turret, w5500::{socket_register::{self, SocketBlock}, W5500}, FLIR_TURRET_PORT};
use arduino_hal::Spi;
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
    let mut serial = arduino_hal::default_serial!(peripherals, pins, 57600);    
    let _ = ufmt::uwriteln!(&mut serial, "Starting Pan Tilt");

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

    arduino_hal::delay_ms(100);
    let (version, _) = W5500::new(Default::default(), GATEWAY, SUBNET, MAC, IP, &mut spi, &mut cs, &mut serial);
    let common_block = W5500::common_register();
    
    let _ = ufmt::uwriteln!(&mut serial, "W5500 Version: {}", version);
    let gateway = common_block.read_gateway_addr(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 Gateway: {:?}", gateway);
    let subnet = common_block.read_subnet(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 Subnet: {:?}", subnet);
    let mac = common_block.read_mac(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 MAC: {:?}", mac);
    let ip = common_block.read_ip(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 Ip: {:?}", ip);
    let rtr = common_block.read_retry_time(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 Retry Time: {:?}", rtr);
    let rcr = common_block.read_retry_count(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 Retry Count: {:?}", rcr);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 Link Status {}", common_block.read_phy_cfg(&mut spi, &mut cs).link_status());

    let mode = socket_register::Mode::default().set_protocol_tcp();
    let mut socket0 = W5500::socket_n(SocketBlock::SOCKET0, mode, FLIR_TURRET_PORT, &mut spi, &mut cs);
    // let mut mainctl = MainCtl::new(socket0);
    
    let pan = StepperMotor::new(pins.d5.into_output(), pins.d4.into_output(), None, None, 200, Some(16), 1000, false);
    let tilt = StepperMotor::new(pins.d3.into_output(), pins.d2.into_output(), None, None, 200, Some(16), 1000, true);


   
    loop{
        if socket0.server_connected(&mut spi, &mut cs, &mut serial){
        }
    }
}

