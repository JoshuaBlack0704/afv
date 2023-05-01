use arduino_hal::{Spi, spi::ChipSelectPin, hal::{port::PB2, usart::Usart0}, clock::MHz16};
use embedded_hal::digital::v2::OutputPin;
use serde::{Serialize, Deserialize};
use ufmt::derive::uDebug;

use crate::{LIGHTS_PORT, network::InternalMessage, w5500::{socket_register::{SocketBlock, self, Socket}, W5500}};

#[derive(uDebug, Serialize, Deserialize, Clone, Copy)]
pub enum LightsMsg{
    TurnOn,
    TurnOff,
}

pub struct Lights<Pin: OutputPin>{
    socket: Socket,
    ctl: Pin,
}

impl<Pin:OutputPin> Lights<Pin>{
    pub fn new(socket_block: SocketBlock, ctl_pin: Pin, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>, serial: &mut Usart0<MHz16>) -> Self {
        let mode = socket_register::Mode::default().set_protocol_tcp();
        let socket = W5500::socket_n(socket_block, mode, LIGHTS_PORT, spi, cs);
        let _ = ufmt::uwriteln!(serial, "Created lights using port {}", LIGHTS_PORT);

        Self{
            socket,
            ctl: ctl_pin,
        }
    }

    pub fn process(&mut self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>, serial: &mut Usart0<MHz16>){
        match self.socket.receive_connected(spi, cs, serial){
            Some(InternalMessage::Lights(LightsMsg::TurnOn)) => {
                let _ = ufmt::uwriteln!(serial, "Lights on");
                let _ = self.ctl.set_high();
            }
            Some(InternalMessage::Lights(LightsMsg::TurnOff)) => {
                let _ = ufmt::uwriteln!(serial, "Lights off");
                let _ = self.ctl.set_low();
            }
            _ => {}
        }
        
    }
}
