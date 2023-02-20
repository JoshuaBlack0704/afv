#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use afv_internal::{w5500::{W5500, socket_register::{SocketStatus, self, Command, SocketBlock}, common_register::ModeRegister}, TESTPORT};
use arduino_hal::Spi;
use embedded_hal::spi::{Polarity, Phase};
use panic_halt as _;

const GATEWAY: [u8;4] = [169,245,228,0];
const SUBNET: [u8;4] = [255,255,255,0];
const MAC: [u8;6] = [0x00,0x08,0xdc,0x01,0x02,0x03];
const IP: [u8;4] = [169,254,228,10];

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
    let (version, w5500) = W5500::new(Default::default(), GATEWAY, SUBNET, MAC, IP, &mut spi, &mut cs);
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
    let mut socket0 = W5500::socket_n(SocketBlock::SOCKET0, mode, TESTPORT, &mut spi, &mut cs);
    let mut msg_count = 0;

    loop {
        if let SocketStatus::Closed = socket0.read_status(&mut spi, &mut cs){
            let _ = ufmt::uwriteln!(&mut serial, "Socket 0 Closed");
            let _ = ufmt::uwriteln!(&mut serial, "Socket 0 Listening");
            socket0.block_listen(&mut spi, &mut cs);
            let _ = ufmt::uwriteln!(&mut serial, "Socket 0 connected to peer at: {:?}", socket0.peer_ip());
            continue;
        }

        socket0.receive(&mut spi, &mut cs);
        if let Some(msg) = socket0.last_msg(){
            let _ = ufmt::uwriteln!(&mut serial, "Socket 0 received msg {}: {:?}", msg_count, msg);
            msg_count += 1;
        }

    }
}
