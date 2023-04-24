use arduino_hal::{spi::ChipSelectPin, hal::{port::PB2, usart::Usart0}, clock::MHz16, Spi};
use embedded_hal::digital::v2::OutputPin;
use serde::{Serialize, Deserialize};
use ufmt::derive::uDebug;

use crate::{w5500::{socket_register::{Socket, SocketBlock, self}, W5500}, PUMP_PORT, network::InternalMessage};

#[derive(uDebug, Serialize, Deserialize, Clone, Copy)]
pub enum PumpMsg{
    TurnOn,
    TurnOff,
}

pub struct Pump<Pin: OutputPin>{
    socket: Socket,
    ctl: Pin,
}

impl<Pin:OutputPin> Pump<Pin>{
    pub fn new(socket_block: SocketBlock, ctl_pin: Pin, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>, serial: &mut Usart0<MHz16>) -> Self {
        let mode = socket_register::Mode::default().set_protocol_tcp();
        let socket = W5500::socket_n(socket_block, mode, PUMP_PORT, spi, cs);
        let _ = ufmt::uwriteln!(serial, "Created pump using port {}", PUMP_PORT);

        Self{
            socket,
            ctl: ctl_pin,
        }
    }

    pub fn process(&mut self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>, serial: &mut Usart0<MHz16>){
        match self.socket.receive_connected(spi, cs, serial){
            Some(InternalMessage::Pump(PumpMsg::TurnOn)) => {
                let _ = ufmt::uwriteln!(serial, "Pump on");
                let _ = self.ctl.set_high();
            }
            Some(InternalMessage::Pump(PumpMsg::TurnOff)) => {
                let _ = ufmt::uwriteln!(serial, "Pump off");
                let _ = self.ctl.set_low();
            }
            _ => {}
        }
        
    }
}
