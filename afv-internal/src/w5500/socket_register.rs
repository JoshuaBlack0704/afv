use arduino_hal::{spi::ChipSelectPin, Spi, hal::port::PB2};
use ufmt::derive::uDebug;

use crate::{network::InternalMessage, SOCKET_MSG_SIZE};

use super::{control::{Bsb, ControlByte, Rw, Om}, W5500, header, read, write};


pub struct SocketAddress(u16);
impl From<SocketAddress> for u16{
    fn from(sa: SocketAddress) -> Self {
        sa.0
    }
}
impl SocketAddress{
    pub const MODE:         Self = Self(0x0000);
    pub const COMMAND:      Self = Self(0x0001);
    pub const INTERRUPT:    Self = Self(0x0002);
    pub const STATUS:       Self = Self(0x0003);
    pub const SOURCE_PORT:  Self = Self(0x0004);
    pub const DST_MAC:      Self = Self(0x0006);
    pub const DST_IP:       Self = Self(0x000c);
    pub const DST_PORT:     Self = Self(0x0010);
    pub const MAX_SEG_SIZE: Self = Self(0x0012);
    pub const IP_TOS:       Self = Self(0x0015);
    pub const IP_TTL:       Self = Self(0x0016);
    pub const RX_BUFF_SIZE: Self = Self(0x001e);
    pub const TX_BUFF_SIZE: Self = Self(0x001f);
    pub const TX_FREE_SIZE: Self = Self(0x0020);
    pub const TX_READ_PTR:  Self = Self(0x0022);
    pub const TX_WRITE_PTR: Self = Self(0x0024);
    pub const RX_RCV_SIZE:  Self = Self(0x0026);
    pub const RX_READ_PTR:  Self = Self(0x0028);
    pub const RX_WRITE_PTR: Self = Self(0x002a);
    pub const INT_MASK:     Self = Self(0x002c);
    pub const FRAG_OFST:    Self = Self(0x002d);
    pub const KEEP_ALV_TMR: Self = Self(0x002f);
}   

pub const MODE_SIZE:usize          = 1;
pub const COMMAND_SIZE:usize       = 1;
pub const INTERRUPT_SIZE:usize     = 1;
pub const STATUS_SIZE:usize        = 1;
pub const SOURCE_PORT_SIZE:usize   = 2;
pub const DST_MAC_SIZE:usize       = 6;
pub const DST_IP_SIZE:usize        = 4;
pub const DST_PORT_SIZE:usize      = 2;
pub const MAX_SEG_SIZE_SIZE:usize  = 2;
pub const IP_TOS_SIZE:usize        = 1;
pub const IP_TTL_SIZE:usize        = 1;
pub const RX_BUFF_SIZE_SIZE:usize  = 1;
pub const TX_BUFF_SIZE_SIZE:usize  = 1;
pub const TX_FREE_SIZE_SIZE:usize  = 2;
pub const TX_READ_PTR_SIZE:usize   = 2;
pub const TX_WRITE_PTR_SIZE:usize  = 2;
pub const RX_RCV_SIZE_SIZE:usize   = 2;
pub const RX_READ_PTR_SIZE:usize   = 2;
pub const RX_WRITE_PTR_SIZE:usize  = 2;
pub const INT_MASK_SIZE:usize      = 1;
pub const FRAG_OFST_SIZE:usize     = 2;
pub const KEEP_ALV_TMR_SIZE:usize  = 1;

pub struct SocketBlock{
    ctl: Bsb,
    tx: Bsb,
    rx: Bsb,
}
impl SocketBlock{
    pub fn socket_ctl(&self) -> Bsb {
        self.ctl
    }
    pub fn socket_tx(&self) -> Bsb {
        self.tx
    }
    pub fn socket_rx(&self) -> Bsb {
        self.rx
    }
    pub const SOCKET0: Self = Self{ctl: Bsb::SOCKET0, tx: Bsb::SOCKET0_TX, rx: Bsb::SOCKET0_RX};
    pub const SOCKET1: Self = Self{ctl: Bsb::SOCKET1, tx: Bsb::SOCKET1_TX, rx: Bsb::SOCKET1_RX};
    pub const SOCKET2: Self = Self{ctl: Bsb::SOCKET2, tx: Bsb::SOCKET2_TX, rx: Bsb::SOCKET2_RX};
    pub const SOCKET3: Self = Self{ctl: Bsb::SOCKET3, tx: Bsb::SOCKET3_TX, rx: Bsb::SOCKET3_RX};
    pub const SOCKET4: Self = Self{ctl: Bsb::SOCKET4, tx: Bsb::SOCKET4_TX, rx: Bsb::SOCKET4_RX};
    pub const SOCKET5: Self = Self{ctl: Bsb::SOCKET5, tx: Bsb::SOCKET5_TX, rx: Bsb::SOCKET5_RX};
    pub const SOCKET6: Self = Self{ctl: Bsb::SOCKET6, tx: Bsb::SOCKET6_TX, rx: Bsb::SOCKET6_RX};
    pub const SOCKET7: Self = Self{ctl: Bsb::SOCKET7, tx: Bsb::SOCKET7_TX, rx: Bsb::SOCKET7_RX};
}

#[derive(Debug, uDebug)]
pub enum SocketStatus{
    Closed,
    Init,
    Listen,
    Established,
    Close,
    Udp,
    Macraw,
    SynSent,
    SynRecv,
    FinWait,
    Closing,
    TimeWait,
    LastAck,
    Unknown,
}
impl From<u8> for SocketStatus{
    fn from(status: u8) -> Self {
        if status == 0x00{
            return Self::Closed;
        }
        if status == 0x13{
            return Self::Init;
        }
        if status == 0x14{
            return Self::Listen;
        }
        if status == 0x17{
            return Self::Established;
        }
        if status == 0x1c{
            return Self::Close;
        }
        if status == 0x22{
            return Self::Udp;
        }
        if status == 0x42{
            return Self::Macraw;
        }
        if status == 0x15{
            return Self::SynSent;
        }
        if status == 0x16{
            return Self::SynRecv;
        }
        if status == 0x18{
            return Self::FinWait;
        }
        if status == 0x1a{
            return Self::Closing;
        }
        if status == 0x1b{
            return Self::TimeWait;
        }
        if status == 0x1d{
            return Self::LastAck;
        }

        Self::Unknown
    }
}

pub struct Mode{
    reg: u8,
}
impl Mode{
    pub fn enable_upd_multicasting(mut self) -> Self {
        self.reg |= 0b10000000;
        self
    }
    pub fn disable_upd_multicasting(mut self) -> Self {
        self.reg &= 0b01111111;
        self
    }
    pub fn enable_broadcast_block(mut self) -> Self {
        self.reg |= 0b10000000;
        self
    }
    pub fn disable_broadcast_block(mut self) -> Self {
        self.reg &= 0b10111111;
        self
    }
    pub fn enable_no_delay_ack(mut self) -> Self {
        self.reg |= 0b00100000;
        self
    }
    pub fn disable_no_delay_ack(mut self) -> Self {
        self.reg &= 0b11011111;
        self
    }
    pub fn enable_unicast(mut self) -> Self {
        self.reg |= 0b00010000;
        self
    }
    pub fn disable_unicast(mut self) -> Self {
        self.reg &= 0b11101111;
        self
    }
    pub fn set_protocol_closed(mut self) -> Self{
        self.reg &= 0b11110000;
        self.reg |= 0b00000000;
        self
    }
    pub fn set_protocol_tcp(mut self) -> Self{
        self.reg &= 0b11110000;
        self.reg |= 0b00000001;
        self
    }
    pub fn set_protocol_udp(mut self) -> Self{
        self.reg &= 0b11110000;
        self.reg |= 0b00000010;
        self
    }
    pub fn set_protocol_macraw(mut self) -> Self{
        self.reg &= 0b11110000;
        self.reg |= 0b00000100;
        self
    }
}
impl Default for Mode{
    fn default() -> Self {
        Self{
            reg: 0,
        }
    }
}
impl From<Mode> for u8{
    fn from(node: Mode) -> Self {
        node.reg
    }
}
impl From<[u8;1]> for Mode{
    fn from(mode: [u8;1]) -> Self {
        Self{
            reg: mode[0]
        }
    }
}

pub struct Command(u8);
impl From<Command> for u8{
    fn from(c: Command) -> Self {
        c.0
    }
}
impl From<[u8;1]> for Command{
    fn from(cmd: [u8;1]) -> Self {
        Command(cmd[0])
    }
}
impl Command{
    pub const OPEN:       Self = Self(0x01);
    pub const LISTEN:     Self = Self(0x02);
    pub const CONNECT:    Self = Self(0x04);
    pub const DISCONNECT: Self = Self(0x08);
    pub const CLOSE:      Self = Self(0x10);
    pub const SEND:       Self = Self(0x20);
    pub const SEND_MAC:   Self = Self(0x21);
    pub const SEND_KEEP:  Self = Self(0x22);
    pub const RECV:       Self = Self(0x40);
}

pub struct BufferSize(u8);
impl From<BufferSize> for u8{
    fn from(s: BufferSize) -> Self {
        s.0
    }
}
impl BufferSize{
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub const TWO: Self = Self(2);
    pub const FOUR: Self = Self(4);
    pub const EIGHT: Self = Self(8);
    pub const SIXTEEN: Self = Self(16);
}

pub struct Socket{
    socket_block: SocketBlock,
    peer_ip: Option<[u8;4]>,
    peer_mac: Option<[u8;6]>,
    peer_port: Option<u16>,
    last_msg: Option<InternalMessage>,
}

impl W5500{
    pub fn socket_n(socket_num: SocketBlock, mode: Mode, port: u16, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> Socket {
        let socket = Socket{
            socket_block: socket_num,
            peer_ip: Default::default(),
            peer_mac: Default::default(),
            peer_port: Default::default(),
            last_msg: Default::default(),
        };
        socket.write_mode(mode, spi, cs);
        socket.write_src_port(port, spi, cs);
        socket
    }
}

impl Socket{
    pub fn init(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        self.write_cmd(Command::OPEN, spi, cs);
        loop{
            if let SocketStatus::Init = self.read_status(spi, cs){
                break;
            }
        }
    }
    pub fn block_listen(&mut self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        if let SocketStatus::Closed = self.read_status(spi, cs){
            self.init(spi, cs);
        }
        
        self.write_cmd(Command::LISTEN, spi, cs);
        
        loop{
            if let SocketStatus::Established = self.read_status(spi, cs){
                let ip = self.read_dst_ip(spi, cs);
                let mac = self.read_dst_mac(spi, cs);
                let port = self.read_dst_port(spi, cs);
                self.peer_ip = Some(ip);
                self.peer_mac = Some(mac);
                self.peer_port = Some(port);
                return;
            }
        }
    }
    pub fn peer_ip(&self) -> Option<[u8; 4]> {
        self.peer_ip.clone()
    }
    pub fn peer_mac(&self) -> Option<[u8; 6]> {
        self.peer_mac.clone()
    }
    pub fn peer_port(&self) -> Option<u16> {
        self.peer_port.clone()
    }
    pub fn last_msg(&self) -> Option<InternalMessage> {
        self.last_msg.clone()
    }
    pub fn receive_blocking(&mut self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> InternalMessage {
        loop{
            if let Some(msg) = self.receive(spi, cs){
               return msg; 
            }
        }
    }
    pub fn receive(&mut self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> Option<InternalMessage>{
        if let SocketStatus::Closed = self.read_status(spi, cs){
            return None;
        }
        if self.read_rx_recv_size(spi, cs) >= SOCKET_MSG_SIZE as u16{
            let data = self.read_rx_buff::<SOCKET_MSG_SIZE>(spi, cs);
            let mut msg: Option<InternalMessage> = None;
            if let Ok((_msg, _)) = serde_json_core::from_slice(&data){
                self.last_msg = Some(_msg); 
                msg = Some(_msg);
            }
            let read_ptr = self.read_rx_read_ptr(spi, cs);
            self.write_rx_read_ptr(read_ptr + SOCKET_MSG_SIZE as u16, spi, cs);
            self.write_cmd(Command::RECV, spi, cs);
            return msg;
        }
        None
    }
    pub fn send(&mut self, data: &[u8], spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        if let SocketStatus::Closed = self.read_status(spi, cs){
            return;
        }
        
    }
    pub fn read_rx_buff<const N: usize>(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> [u8;N]{
        let header = header(self.read_rx_read_ptr(spi, cs), ControlByte::new(self.socket_block.socket_rx(), Rw::READ, Om::VDM));
        read::<N>(header, spi, cs)
    }
    pub fn read_mode(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> Mode{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::MODE, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        read::<MODE_SIZE>(header, spi, cs).into()
        
    }
    pub fn write_mode(&self, mode: Mode, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::MODE, ControlByte::new(self.socket_block.socket_ctl(), Rw::WRITE, Om::VDM));
        let mode = [mode.into()];
        write(header, &mode, spi, cs);
    }
    pub fn read_cmd(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> Command{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::COMMAND, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        read::<COMMAND_SIZE>(header, spi, cs).into()
        
    }
    pub fn write_cmd(&self, cmd: Command, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::COMMAND, ControlByte::new(self.socket_block.socket_ctl(), Rw::WRITE, Om::VDM));
        let cmd = [cmd.into()];
        write(header, &cmd, spi, cs);
    }
    pub fn read_status(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> SocketStatus{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::STATUS, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        read::<STATUS_SIZE>(header, spi, cs)[0].into()
    }
    pub fn read_src_port(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u16{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::SOURCE_PORT, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        u16::from_be_bytes(read::<SOURCE_PORT_SIZE>(header, spi, cs))
    }
    pub fn write_src_port(&self, port: impl Into<u16>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::SOURCE_PORT, ControlByte::new(self.socket_block.socket_ctl(), Rw::WRITE, Om::VDM));
        let port = port.into().to_be_bytes();
        write(header, &port, spi, cs);
    }
    pub fn read_dst_mac(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> [u8;DST_MAC_SIZE]{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::DST_MAC, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        read::<DST_MAC_SIZE>(header, spi, cs)
    }
    pub fn write_dst_mac(&self, mac: impl Into<[u8;DST_MAC_SIZE]>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::DST_MAC, ControlByte::new(self.socket_block.socket_ctl(), Rw::WRITE, Om::VDM));
        let mac = mac.into();
        write(header, &mac, spi, cs);
    }
    pub fn read_dst_ip(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> [u8;DST_IP_SIZE]{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::DST_IP, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        read::<DST_IP_SIZE>(header, spi, cs)
    }
    pub fn write_dst_ip(&self, ip: impl Into<[u8;DST_IP_SIZE]>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::DST_IP, ControlByte::new(self.socket_block.socket_ctl(), Rw::WRITE, Om::VDM));
        let ip = ip.into();
        write(header, &ip, spi, cs);
    }
    pub fn read_dst_port(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u16{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::DST_PORT, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        u16::from_be_bytes(read::<DST_PORT_SIZE>(header, spi, cs))
    }
    pub fn write_dst_port(&self, port: impl Into<u16>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::DST_PORT, ControlByte::new(self.socket_block.socket_ctl(), Rw::WRITE, Om::VDM));
        let port = port.into().to_be_bytes();
        write(header, &port, spi, cs);
    }
    pub fn read_rx_buff_size(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u8{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::RX_BUFF_SIZE, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        read::<RX_BUFF_SIZE_SIZE>(header, spi, cs)[0]
    }
    pub fn write_rx_buff_size(&self, size: BufferSize, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::RX_BUFF_SIZE, ControlByte::new(self.socket_block.socket_ctl(), Rw::WRITE, Om::VDM));
        let size = [size.into()];
        write(header, &size, spi, cs);
    }
    pub fn read_tx_buff_size(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u8{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::TX_BUFF_SIZE, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        read::<TX_BUFF_SIZE_SIZE>(header, spi, cs)[0]
    }
    pub fn write_tx_buff_size(&self, size: BufferSize, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::TX_BUFF_SIZE, ControlByte::new(self.socket_block.socket_ctl(), Rw::WRITE, Om::VDM));
        let size = [size.into()];
        write(header, &size, spi, cs);
    }
    pub fn read_tx_free_size(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u16{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::TX_FREE_SIZE, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        u16::from_be_bytes(read::<TX_FREE_SIZE_SIZE>(header, spi, cs))
    }
    pub fn read_tx_read_ptr(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u16{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::TX_READ_PTR, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        u16::from_be_bytes(read::<TX_READ_PTR_SIZE>(header, spi, cs))
    }
    pub fn read_tx_write_ptr(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u16{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::TX_WRITE_PTR, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        u16::from_be_bytes(read::<TX_WRITE_PTR_SIZE>(header, spi, cs))
    }
    pub fn write_tx_write_ptr(&self, ptr: impl Into<u16>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::TX_WRITE_PTR, ControlByte::new(self.socket_block.socket_ctl(), Rw::WRITE, Om::VDM));
        let ptr = ptr.into().to_be_bytes();
        write(header, &ptr, spi, cs);
    }
    pub fn read_rx_recv_size(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u16{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::RX_RCV_SIZE, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        u16::from_be_bytes(read::<RX_RCV_SIZE_SIZE>(header, spi, cs))
    }
    pub fn read_rx_read_ptr(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u16{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::RX_READ_PTR, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        u16::from_be_bytes(read::<RX_READ_PTR_SIZE>(header, spi, cs))
    }
    pub fn write_rx_read_ptr(&self, ptr: impl Into<u16>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::RX_READ_PTR, ControlByte::new(self.socket_block.socket_ctl(), Rw::WRITE, Om::VDM));
        let ptr = ptr.into().to_be_bytes();
        write(header, &ptr, spi, cs);
    }
    pub fn read_rx_write_ptr(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u16{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::RX_WRITE_PTR, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        u16::from_be_bytes(read::<RX_WRITE_PTR_SIZE>(header, spi, cs))
    }
    pub fn read_keep_alive(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u8{
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::KEEP_ALV_TMR, ControlByte::new(self.socket_block.socket_ctl(), Rw::READ, Om::VDM));
        read::<KEEP_ALV_TMR_SIZE>(header, spi, cs)[0]
    }
    pub fn write_keep_alive(&self, timer: impl Into<u8>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>){
        arduino_hal::delay_us(1);
        let header = header(SocketAddress::KEEP_ALV_TMR, ControlByte::new(self.socket_block.socket_ctl(), Rw::WRITE, Om::VDM));
        let timer = [timer.into()];
        write(header, &timer, spi, cs);
    }
    
}
