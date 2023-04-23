use arduino_hal::{spi::ChipSelectPin, hal::{port::PB2, usart::Usart0}, Spi, clock::MHz16};

use crate::{w5500::socket_register::Socket, network::InternalMessage};

pub const PUMP_REFRESH_BUDGET: u32 = 3000;

pub struct MainCtl{
    socket: Socket,
    server_connected: bool,
}

impl MainCtl{
    
    pub fn new(socket: Socket) -> MainCtl {
        Self{
            socket,
            server_connected: false,
        }
    }
    /// Will update and conduct all internal systems in a non-blocking manner
    pub fn process(&mut self, serial: &mut Usart0<MHz16>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> bool{
        // let _ = ufmt::uwriteln!(serial, "MAIN CTL: Server Status {:?}", self.socket.read_status(spi, cs));
        if self.socket.server_connected(spi, cs, serial){
            if !self.server_connected{
                let _ = ufmt::uwriteln!(serial, "MAIN CTL: Server connected");
                self.server_connected = true;
            }
            if let Some(msg) = self.socket.receive(spi, cs, serial){
                match msg{
                    InternalMessage::Ping(p) => {
                        let _ = ufmt::uwriteln!(serial, "MAIN CTL: pinged with {}", p);
                        // self.socket.send(msg, spi, cs);
                        return true;
                    },
                    InternalMessage::FlirSignatureOffset(_) => todo!(),
                    _ => {},
                }
            }
        }
        else{
            if self.server_connected{
                let _ = ufmt::uwriteln!(serial, "MAIN CTL: Server disconnected");
                self.server_connected = false;
            }
        }
        false
    }
}