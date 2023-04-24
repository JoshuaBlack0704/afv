use arduino_hal::{Spi, spi::ChipSelectPin, hal::{port::PB2, usart::Usart0}, clock::MHz16};
use embedded_hal::digital::v2::OutputPin;
use serde::{Serialize, Deserialize};
use ufmt::derive::uDebug;

use crate::{network::InternalMessage, w5500::{socket_register::{Socket, SocketBlock, self}, W5500}, SIREN_PORT};

#[derive(uDebug, Serialize, Deserialize, Clone, Copy)]
pub enum SirenMsg{
    TurnOn,
    TurnOff,
}

pub struct Siren<Pin: OutputPin>{
    socket: Socket,
    ctl: Pin,
}

impl<Pin:OutputPin> Siren<Pin>{
    pub fn new(socket_block: SocketBlock, ctl_pin: Pin, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>, serial: &mut Usart0<MHz16>) -> Self {
        let mode = socket_register::Mode::default().set_protocol_tcp();
        let socket = W5500::socket_n(socket_block, mode, SIREN_PORT, spi, cs);
        let _ = ufmt::uwriteln!(serial, "Created siren using port {}", SIREN_PORT);

        Self{
            socket,
            ctl: ctl_pin,
        }
    }

    pub fn process(&mut self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>, serial: &mut Usart0<MHz16>){
        match self.socket.receive_connected(spi, cs, serial){
            Some(InternalMessage::Siren(SirenMsg::TurnOn)) => {
                let _ = ufmt::uwriteln!(serial, "Siren on");
                let _ = self.ctl.set_high();
            }
            Some(InternalMessage::Siren(SirenMsg::TurnOff)) => {
                let _ = ufmt::uwriteln!(serial, "Siren off");
                let _ = self.ctl.set_low();
            }
            _ => {}
        }
        
    }
}
