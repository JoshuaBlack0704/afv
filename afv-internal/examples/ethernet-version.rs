#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use afv_internal::{w5500::{W5500, socket_register::{self, SocketBlock}}, mainctl::MainCtl};
use afv_internal::FLIR_TURRET_PORT;
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
    let socket0 = W5500::socket_n(SocketBlock::SOCKET0, mode, FLIR_TURRET_PORT, &mut spi, &mut cs);
    let mut mainctl = MainCtl::new(socket0);

    let mut pin5 = pins.d5.into_output();
    let mut pin4 = pins.d4.into_output();
    pin4.set_low();
    let mut pin3 = pins.d3.into_output();
    let mut pin2 = pins.d2.into_output();
    pin2.set_low();

    loop {
            for _ in  0..200{
                arduino_hal::delay_ms(1);
                for _ in 0..16{
                    pin5.set_high();
                    pin3.set_high();
                    arduino_hal::delay_us(20);
                    pin5.set_low();
                    pin3.set_low();
                    arduino_hal::delay_us(20);
                }
            }
        if mainctl.process(&mut serial, &mut spi, &mut cs){
        }
    }
}
