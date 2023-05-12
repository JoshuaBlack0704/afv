use arduino_hal::{
    clock::MHz16,
    hal::{port::PB2, usart::Usart0},
    spi::ChipSelectPin,
    I2c, Spi,
};
use serde::{Deserialize, Serialize};
use ufmt::derive::uDebug;

use crate::{
    network::InternalMessage,
    w5500::{
        socket_register::{self, Socket, SocketBlock},
        W5500,
    },
    LIDAR_PORT,
};

#[derive(uDebug, Serialize, Deserialize, Clone, Copy)]
pub enum LidarMsg {
    PollLidar,
    LidarDistanceCm(u32),
}

pub trait I2cLidarOps {
    fn read_distance_cm(&mut self, i2c: &mut I2c, serial: &mut Usart0<MHz16>) -> u16;
}

pub struct Lidar<L: I2cLidarOps> {
    socket: Socket,
    lidar: L,
}

impl<L: I2cLidarOps> Lidar<L> {
    pub fn new(
        socket_block: SocketBlock,
        lidar: L,
        spi: &mut Spi,
        cs: &mut ChipSelectPin<PB2>,
        serial: &mut Usart0<MHz16>,
    ) -> Self {
        let mode = socket_register::Mode::default().set_protocol_tcp();
        let socket = W5500::socket_n(socket_block, mode, LIDAR_PORT, spi, cs);
        let _ = ufmt::uwriteln!(serial, "Created lidar using port {}", LIDAR_PORT);
        Self { socket, lidar }
    }

    pub fn process(
        &mut self,
        i2c: &mut I2c,
        spi: &mut Spi,
        cs: &mut ChipSelectPin<PB2>,
        serial: &mut Usart0<MHz16>,
    ) {
        if let Some(msg) = self.socket.receive_connected(spi, cs, serial) {
            match msg {
                InternalMessage::Ping(_) => {
                    self.socket.send(msg, spi, cs);
                }
                InternalMessage::Lidar(LidarMsg::PollLidar) => {
                    // let _ = ufmt::uwriteln!(serial, "Lidar distance polled");
                    self.poll_distance(i2c, spi, cs, serial);
                }
                _ => {}
            }
        }
    }
    pub fn poll_distance(
        &mut self,
        i2c: &mut I2c,
        spi: &mut Spi,
        cs: &mut ChipSelectPin<PB2>,
        serial: &mut Usart0<MHz16>,
    ) {
        let distance = self.lidar.read_distance_cm(i2c, serial);
        // let _ = ufmt::uwriteln!(serial, "Lidar calculated distance {}", distance);
        let msg = InternalMessage::Lidar(LidarMsg::LidarDistanceCm(
            self.lidar.read_distance_cm(i2c, serial) as u32,
        ));
        self.socket.send(msg, spi, cs);
        // let _ = ufmt::uwriteln!(serial, "Lidar sent distance");
    }
}
