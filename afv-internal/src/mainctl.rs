use arduino_hal::{spi::ChipSelectPin, hal::{port::PB2, usart::Usart0}, Spi, clock::MHz16};

use crate::{w5500::socket_register::Socket, network::InternalMessage};

pub const PUMP_REFRESH_BUDGET: u32 = 3000;

pub struct MainCtl{
    socket: Socket,
    pump_refresh_budget: u32,
    pump_status: bool,
    server_connected: bool,
}

impl MainCtl{
    pub fn new(socket: Socket) -> MainCtl {
        Self{
            socket,
            pump_status: Default::default(),
            pump_refresh_budget: PUMP_REFRESH_BUDGET,
            server_connected: false,
        }
    }
    /// Will update and conduct all internal systems in a non-blocking manner
    pub fn process(&mut self, serial: &mut Usart0<MHz16>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        // let _ = ufmt::uwriteln!(serial, "MAIN CTL: Server Status {:?}", self.socket.read_status(spi, cs));
        if self.socket.server_connected(spi, cs){
            if !self.server_connected{
                let _ = ufmt::uwriteln!(serial, "MAIN CTL: Server connected");
                self.server_connected = true;
            }
            if let Some(msg) = self.socket.receive(spi, cs){
                match msg{
                    InternalMessage::Ping(p) => {
                        let _ = ufmt::uwriteln!(serial, "MAIN CTL: pinged with {}", p);
                        self.socket.send(msg, spi, cs);
                    },
                    InternalMessage::PumpState(state) => {
                        if state {
                            self.pump_refresh_budget = PUMP_REFRESH_BUDGET;
                            if !self.pump_status{
                                let _ = ufmt::uwriteln!(serial, "MAIN CTL: pump activatd");
                                self.pump_status = true;
                            }
                        }
                        else{
                            self.pump_refresh_budget = 0;
                        }
                    },
                }
            }
        }
        else{
            if self.server_connected{
                let _ = ufmt::uwriteln!(serial, "MAIN CTL: Server disconnected");
                self.server_connected = false;
            }
        }



        if self.pump_refresh_budget > 0{
            self.pump_refresh_budget -= 1;
        }
        else{
            if self.pump_status{
                let _ = ufmt::uwriteln!(serial, "MAIN CTL: pump deactivated");
                self.pump_status = false;
            }
        }
    }
}