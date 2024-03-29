use arduino_hal::{
    clock::MHz16,
    hal::{port::PB2, usart::Usart0},
    prelude::{_embedded_hal_blocking_spi_Write, _embedded_hal_spi_FullDuplex},
    spi::ChipSelectPin,
    Spi,
};
use embedded_hal::digital::v2::OutputPin;

pub mod common_register;
pub mod control;
pub mod socket_register;

pub struct W5500 {}
impl W5500 {
    pub fn new(
        mode: common_register::ModeRegister,
        gateway: [u8; 4],
        subnet: [u8; 4],
        mac: [u8; 6],
        ip: [u8; 4],
        spi: &mut Spi,
        cs: &mut ChipSelectPin<PB2>,
        serial: &mut Usart0<MHz16>,
    ) -> (u8, W5500) {
        let common = Self::common_register();
        common.write_mode_register(mode, spi, cs);
        arduino_hal::delay_us(1);
        common.write_gateway_addr(gateway, spi, cs);
        arduino_hal::delay_us(1);
        common.write_subnet(subnet, spi, cs);
        arduino_hal::delay_us(1);
        common.write_mac(mac, spi, cs);
        arduino_hal::delay_us(1);
        common.write_ip(ip, spi, cs);
        arduino_hal::delay_us(1);
        let version = common.read_version_register(spi, cs);
        let _ = ufmt::uwriteln!(serial, "W5500 Version: {}", version);
        let gateway = common.read_gateway_addr(spi, cs);
        let _ = ufmt::uwriteln!(serial, "W5500 Gateway: {:?}", gateway);
        let subnet = common.read_subnet(spi, cs);
        let _ = ufmt::uwriteln!(serial, "W5500 Subnet: {:?}", subnet);
        let mac = common.read_mac(spi, cs);
        let _ = ufmt::uwriteln!(serial, "W5500 MAC: {:?}", mac);
        let ip = common.read_ip(spi, cs);
        let _ = ufmt::uwriteln!(serial, "W5500 Ip: {:?}", ip);
        let rtr = common.read_retry_time(spi, cs);
        let _ = ufmt::uwriteln!(serial, "W5500 Retry Time: {:?}", rtr);
        let rcr = common.read_retry_count(spi, cs);
        let _ = ufmt::uwriteln!(serial, "W5500 Retry Count: {:?}", rcr);
        let _ = ufmt::uwriteln!(
            serial,
            "W5500 Link Status {}",
            common.read_phy_cfg(spi, cs).link_status()
        );
        (common.read_version_register(spi, cs), Self {})
    }
}

pub fn header(addr: impl Into<u16>, control: impl Into<u8>) -> [u8; 3] {
    let addr = addr.into().to_be_bytes();
    let control = control.into();
    [addr[0], addr[1], control]
}

pub fn read<const N: usize>(
    header: impl Into<[u8; 3]>,
    spi: &mut Spi,
    cs: &mut ChipSelectPin<PB2>,
) -> [u8; N] {
    let header = header.into();
    let mut data = [0u8; N];
    let _ = cs.set_low();
    let _ = spi.write(&header);
    for i in 0..N {
        let _ = spi.write(&[0]);
        data[i] = spi.read().unwrap();
    }
    let _ = cs.set_high();
    data
}
pub fn write(header: impl Into<[u8; 3]>, data: &[u8], spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
    let header = header.into();
    let _ = cs.set_low();
    let _ = spi.write(&header);
    let _ = spi.write(data);
    let _ = cs.set_high();
}
