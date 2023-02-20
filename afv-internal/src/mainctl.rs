use arduino_hal::{spi::ChipSelectPin, hal::{port::PB2, usart::Usart0}, Spi, clock::MHz16};

use crate::{w5500::socket_register::Socket, network::InternalMessage};

pub const PUMP_REFRESH_BUDGET: u32 = 16000000;

pub struct MainCtl{
    socket: Socket,
    pump_refresh_budget: u32,
    pump_status: bool,
}

impl MainCtl{
    pub fn new(socket: Socket) -> MainCtl {
        Self{
            socket,
            pump_status: Default::default(),
            pump_refresh_budget: PUMP_REFRESH_BUDGET,
        }
    }
    /// Will update and conduct all internal systems in a non-blocking manner
    pub fn process(&mut self, serial: &mut Usart0<MHz16>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        if let Some(msg) = self.socket.receive(spi, cs){
            match msg{
                InternalMessage::Ping(p) => {
                    let _ = ufmt::uwriteln!(serial, "MAIN CTL: pinged with {}", p);
                },
                InternalMessage::ActivatePump => {
                    self.pump_refresh_budget = PUMP_REFRESH_BUDGET;
                    if !self.pump_status{
                        let _ = ufmt::uwriteln!(serial, "MAIN CTL: pump activatd");
                        self.pump_status = true;
                    }
                },
            }
        }


        if self.pump_refresh_budget > 0{
            self.pump_refresh_budget -= 1;
        }
        else{
            if self.pump_status{
                let _ = ufmt::uwriteln!(serial, "MAIN CTL: pump deactivated");
            }
        }
    }
}