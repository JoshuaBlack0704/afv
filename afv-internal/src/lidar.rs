use arduino_hal::{spi::ChipSelectPin, hal::{usart::Usart0, port::PB2}, clock::MHz16, Spi};
use serde::{Serialize, Deserialize};
use ufmt::derive::uDebug;

use crate::{w5500::{socket_register::{Socket, SocketBlock, self}, W5500}, LIDAR_PORT, network::InternalMessage};

#[derive(uDebug, Serialize, Deserialize, Clone, Copy)]
pub enum LidarMsg{
    PollLidar,
    LidarDistanceCm(u32),
}

pub struct Lidar{
    socket: Socket,
    socket_connected: bool,
}

impl Lidar{
    pub fn new(socket_block: SocketBlock, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>, serial: &mut Usart0<MHz16>) -> Self{
        let mode = socket_register::Mode::default().set_protocol_tcp();
        let socket = W5500::socket_n(socket_block, mode, LIDAR_PORT, spi, cs);
        let _ = ufmt::uwriteln!(serial, "Created lidar using port {}", LIDAR_PORT);
        Self{
           socket,
            socket_connected: true 
        }
    }

    pub fn process(&mut self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>, serial: &mut Usart0<MHz16>){
        // let _ = ufmt::uwriteln!(serial, "Turret proccessing");
        if self.socket.server_connected(spi, cs, serial){
            if !self.socket_connected{
                let _ = ufmt::uwriteln!(serial, "Lidar connected");
                self.socket_connected = true;
            }
            if let Some(msg) = self.socket.receive(spi, cs, serial){
                match msg{
                    InternalMessage::Ping(_) => {
                        self.socket.send(msg, spi, cs);
                    },
                    InternalMessage::Lidar(LidarMsg::PollLidar) => {
                        let _ = ufmt::uwriteln!(serial, "Lidar distance polled");
                        self.poll_distance(spi, cs, serial);
                    }
                    _ => {},
                }
            }
        }
        else{
            if self.socket_connected{
                let _ = ufmt::uwriteln!(serial, "Lidar disconnected");
                self.socket_connected= false;
            }
        }
    }
    fn poll_distance(&mut self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>, serial: &mut Usart0<MHz16>){
        let _ = ufmt::uwriteln!(serial, "Lidar calculated steps");
        let msg = InternalMessage::Lidar(LidarMsg::LidarDistanceCm(4000));
        self.socket.send(msg, spi, cs);
        let _ = ufmt::uwriteln!(serial, "Lidar sent distance");
    }
}