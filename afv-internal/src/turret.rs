use arduino_hal::{clock::MHz16, hal::{usart::Usart0, port::PB2}, Spi, spi::ChipSelectPin};
use serde::{Serialize, Deserialize};
use ufmt::derive::uDebug;

use crate::{stepper::{StepperOps, StepperOpsError}, w5500::{socket_register::{SocketBlock, self, Socket}, W5500}, network::InternalMessage};

#[derive(uDebug, Serialize, Deserialize, Clone, Copy)]
pub enum TurretMsg{
    PollSteps,
    SetSteps(i32, i32),
    Steps((i32, i32)),
}

/// Zero is at direct forward
/// Left is [-max_degrees, 0]
/// Right is [0, max_degrees]
pub struct Turret<PS: StepperOps, TS: StepperOps>{
    port: u16,
    socket: Socket,
    socket_connected: bool,
    pan_stepper: PS,
    tilt_stepper: TS,
} 

impl<PS:StepperOps, TS: StepperOps> Turret<PS, TS>{
    pub fn new(pan_stepper: PS, tilt_stepper: TS, port: u16, socket_block: SocketBlock, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>, serial: &mut Usart0<MHz16>) -> Self {
        let mode = socket_register::Mode::default().set_protocol_tcp();
        let socket = W5500::socket_n(socket_block, mode, port, spi, cs);
        let _ = ufmt::uwriteln!(serial, "Created turret using port {}", port);
        Self{
            pan_stepper,
            tilt_stepper,
            socket,
            socket_connected: true,
            port,
        }
    }
    pub fn process(&mut self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>, serial: &mut Usart0<MHz16>){
        // let _ = ufmt::uwriteln!(serial, "Turret proccessing");
        if self.socket.server_connected(spi, cs, serial){
            if !self.socket_connected{
                let _ = ufmt::uwriteln!(serial, "Turret {} connected", self.port);
                self.socket_connected = true;
            }
            if let Some(msg) = self.socket.receive(spi, cs, serial){
                match msg{
                    InternalMessage::Ping(_) => {
                        self.socket.send(msg, spi, cs);
                    },
                    InternalMessage::Turret(msg) => {
                        match msg{
                            TurretMsg::PollSteps => {
                                let _ = ufmt::uwriteln!(serial, "Turret {} steps polled", self.port);
                                self.poll_steps(spi, cs, serial)
                            },
                            TurretMsg::SetSteps(_, _) => {},
                            _ => {}
                        }
                        
                    },
                    _ => {},
                }
            }
        }
        else{
            if self.socket_connected{
                let _ = ufmt::uwriteln!(serial, "Turret {} disconnected", self.port);
                self.socket_connected= false;
            }
        }
    }
    fn poll_steps(&mut self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>, serial: &mut Usart0<MHz16>){
        let pan_steps = self.pan_stepper.current_step();
        let tilt_steps = self.tilt_stepper.current_step();


        let _ = ufmt::uwriteln!(serial, "Turret {} calculated steps", self.port);
        let msg = InternalMessage::Turret(TurretMsg::Steps((pan_steps, tilt_steps)));
        self.socket.send(msg, spi, cs);
        let _ = ufmt::uwriteln!(serial, "Turret {} sent steps", self.port);
    }
    /// This algorithim is based off of hard stopping occuring on the turret
    pub fn home(&mut self, serial: &mut Usart0<MHz16>, home_steps: i32) -> Result<(), StepperOpsError>{
        self.pan_stepper.step(home_steps, true, serial)?;
        
        if let Err(_) = self.pan_stepper.to_step(0, serial){
            let _ = ufmt::uwriteln!(serial, "Pan Homing Error");
        }
        
        self.tilt_stepper.step(home_steps, true, serial)?;
        
        if let Err(_) = self.tilt_stepper.to_step(0, serial){
            let _ = ufmt::uwriteln!(serial, "Pan Homing Error");
        }
        

        let _ = ufmt::uwriteln!(serial, "Pan tilt homed");
        Ok(())
    }
}