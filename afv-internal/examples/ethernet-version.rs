#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use afv_internal::w5500::{W5500, socket_register::{SocketN, SocketStatus}};
use arduino_hal::Spi;
use embedded_hal::spi::{Polarity, Phase};
use panic_halt as _;

const GATEWAY: [u8;4] = [10,192,138,254];
const SUBNET: [u8;4] = [255,255,255,0];
const MAC: [u8;6] = [0x00,0x08,0xdc,0x01,0x02,0x03];
const IP: [u8;4] = [10,192,138,13];

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


    let common_block = W5500::common_register();
    let version = common_block.read_version_register(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 Version: {}", version);
    common_block.write_gateway_addr(GATEWAY, &mut spi, &mut cs);    
    arduino_hal::delay_us(10);
    let gateway = common_block.read_gateway_addr(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 Gateway: {:?}", gateway);
    common_block.write_subnet(SUBNET, &mut spi, &mut cs);
    arduino_hal::delay_us(10);
    let subnet = common_block.read_subnet(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 Subnet: {:?}", subnet);
    common_block.write_mac(MAC, &mut spi, &mut cs);
    arduino_hal::delay_us(10);
    let mac = common_block.read_mac(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 MAC: {:?}", mac);
    common_block.write_ip(IP, &mut spi, &mut cs);
    arduino_hal::delay_us(10);
    let ip = common_block.read_ip(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 Ip: {:?}", ip);
    arduino_hal::delay_us(10);
    let rtr = common_block.read_retry_time(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 Retry Time: {:?}", rtr);
    arduino_hal::delay_us(10);
    let rcr = common_block.read_retry_count(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "W5500 Retry Count: {:?}", rcr);

    let socket0 = W5500::socket_n(SocketN::SOCKET0);
    let mut sock_status = SocketStatus::Init;
    sock_status = socket0.read_status(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "Socket 0 Status : {:?}", sock_status);
    socket0.write_src_port(4040u16, &mut spi, &mut cs);
    arduino_hal::delay_us(10);
    let port = socket0.read_src_port(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "Socket 0 Port : {:?}", port);
    let rx_size = socket0.read_rx_buff_size(&mut spi, &mut cs);
    let _ = ufmt::uwriteln!(&mut serial, "Socket 0 Rx Size: {:?}", rx_size);
    
    
    
    




    loop {
    }
}
