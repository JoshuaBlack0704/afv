use arduino_hal::{
    clock::MHz16,
    hal::{port::PB2, usart::Usart0},
    spi::ChipSelectPin,
    Spi,
};
use serde::{Deserialize, Serialize};
use ufmt::derive::uDebug;

use crate::{
    network::InternalMessage,
    stepper::StepperOps,
    w5500::{
        socket_register::{self, Socket, SocketBlock},
        W5500,
    },
};

pub const MAX_PAN_ANGLE: f32 = 100.0;

#[derive(uDebug, Serialize, Deserialize, Clone, Copy)]
pub enum TurretMsg {
    PollSteps,
    SetSteps((i32, i32)),
    Steps((i32, i32)),
}

/// Zero is at direct forward
/// Left is [-max_degrees, 0]
/// Right is [0, max_degrees]
#[allow(unused)]
pub struct Turret<PS: StepperOps, TS: StepperOps> {
    port: u16,
    socket: Socket,
    pan_stepper: PS,
    tilt_stepper: TS,
}

impl<PS: StepperOps, TS: StepperOps> Turret<PS, TS> {
    pub fn new(
        pan_stepper: PS,
        tilt_stepper: TS,
        port: u16,
        socket_block: SocketBlock,
        spi: &mut Spi,
        cs: &mut ChipSelectPin<PB2>,
        serial: &mut Usart0<MHz16>,
    ) -> Self {
        let mode = socket_register::Mode::default().set_protocol_tcp();
        let socket = W5500::socket_n(socket_block, mode, port, spi, cs);
        // let _ = ufmt::uwriteln!(serial, "Created turret using port {}", port);
        Self {
            pan_stepper,
            tilt_stepper,
            socket,
            port,
        }
    }
    pub fn process(
        &mut self,
        spi: &mut Spi,
        cs: &mut ChipSelectPin<PB2>,
        serial: &mut Usart0<MHz16>,
    ) {
        if let Some(msg) = self.socket.receive_connected(spi, cs, serial) {
            match msg {
                InternalMessage::Ping(_) => {
                    self.socket.send(msg, spi, cs);
                }
                InternalMessage::Turret(msg) => {
                    match msg {
                        TurretMsg::PollSteps => {
                            // let _ = ufmt::uwriteln!(serial, "Turret {} steps polled", self.port);
                            self.poll_steps(spi, cs, serial)
                        }
                        TurretMsg::SetSteps(msg) => {
                            let _ = ufmt::uwriteln!(serial, "Turret {} steps set", self.port);
                            self.set_steps(msg, serial);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
    fn poll_steps(
        &mut self,
        spi: &mut Spi,
        cs: &mut ChipSelectPin<PB2>,
        serial: &mut Usart0<MHz16>,
    ) {
        let pan_steps = self.pan_stepper.current_step();
        let tilt_steps = self.tilt_stepper.current_step();

        let msg = InternalMessage::Turret(TurretMsg::Steps((pan_steps, tilt_steps)));
        self.socket.send(msg, spi, cs);
        // let _ = ufmt::uwriteln!(serial, "Turret {} sent steps", self.port);
    }
    fn set_steps(&mut self, steps: (i32, i32), serial: &mut Usart0<MHz16>) {
        let _ = self.pan_stepper.to_step(steps.0, true, serial);
        let _ = self.tilt_stepper.to_step(steps.1, true, serial);
        // let _ = ufmt::uwriteln!(serial, "Turret {} direction canged", self.port);
    }
}
