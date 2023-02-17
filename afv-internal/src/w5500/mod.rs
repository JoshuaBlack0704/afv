use arduino_hal::{Spi, spi::ChipSelectPin, prelude::{_embedded_hal_blocking_spi_Write, _embedded_hal_spi_FullDuplex}, hal::port::PB2};
use embedded_hal::digital::v2::OutputPin;

pub mod control;
pub mod common_offsets;
pub mod socket_offsets;

pub mod common_register;
pub mod socket_register;

pub struct W5500{}

pub fn header(addr: impl Into<u16>, control: impl Into<u8>) -> [u8;3]{
    let addr = addr.into().to_be_bytes();
    let control = control.into();
    [addr[0], addr[1], control]
}

pub fn read<const N:usize>(header: impl Into<[u8;3]>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> [u8; N]{
    let header = header.into();
    let mut data = [0u8;N];
    let _ = cs.set_low();
    let _ = spi.write(&header);
    for i in 0..N{
        let _ = spi.write(&[0]);
        data[i] = spi.read().unwrap();
    }
    let _ = cs.set_high();
    data
}
pub fn write(header: impl Into<[u8;3]>, data: &[u8], spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
    let header = header.into();
    let _ = cs.set_low();
    let _ = spi.write(&header);
    let _ = spi.write(data);
    let _ = cs.set_high();
}