use arduino_hal::{Spi, spi::ChipSelectPin, hal::port::PB2};


use super::{control::{Bsb, ControlByte, Rw, Om}, W5500, header, read, write};

pub const COMMON_BLOCK: Bsb = Bsb::COMMON;

pub struct CommonAddress(u16);
impl From<CommonAddress> for u16{
    fn from(ca: CommonAddress) -> Self {
        ca.0
    }
}
impl CommonAddress{
    pub const MODE:              Self = Self(0x0000);
    pub const GATEWAY:           Self = Self(0x0001);
    pub const SUBNET_MASK:       Self = Self(0x0005);
    pub const SOURCE_MAC:        Self = Self(0x0009);
    pub const SOURCE_IP:         Self = Self(0x000f);
    pub const INTLLT:            Self = Self(0x0013);
    pub const INTERRUPT:         Self = Self(0x0015);
    pub const INT_MASK:          Self = Self(0x0016);
    pub const SOCKET_INT:        Self = Self(0x0017);
    pub const SOCKET_INT_MASK:   Self = Self(0x0018);
    pub const RETRY_TIME:        Self = Self(0x0019);
    pub const RETRY_COUNT:       Self = Self(0x001b);
    pub const PPPLCP_REQ_TIMER:  Self = Self(0x001c);
    pub const PPP_LCP_MAGIC_NUM: Self = Self(0x001d);
    pub const PPP_DST_MAC:       Self = Self(0x001e);
    pub const PPP_SESS_ID:       Self = Self(0x0024);
    pub const PPP_MAX_SEG_SIZE:  Self = Self(0x0026);
    pub const UNREACH_IP:        Self = Self(0x0028);
    pub const UNREACH_PORT:      Self = Self(0x002c);
    pub const PHY_CFG:           Self = Self(0x002e);
    pub const VERSION:           Self = Self(0x0039);
}

pub const MODE_SIZE:usize              = 1;
pub const GATEWAY_SIZE:usize           = 4;
pub const SUBNET_MASK_SIZE:usize       = 4;
pub const SOURCE_MAC_SIZE:usize        = 6;
pub const SOURCE_IP_SIZE:usize         = 4;
pub const INTLLT_SIZE:usize            = 2;
pub const INTERRUPT_SIZE:usize         = 1;
pub const INT_MASK_SIZE:usize          = 1;
pub const SOCKET_INT_SIZE:usize        = 1;
pub const SOCKET_INT_MASK_SIZE:usize   = 1;
pub const RETRY_TIME_SIZE:usize        = 2;
pub const RETRY_COUNT_SIZE:usize       = 1;
pub const PPPLCP_REQ_TIMER_SIZE:usize  = 1;
pub const PPP_LCP_MAGIC_NUM_SIZE:usize = 1;
pub const PPP_DST_MAC_SIZE:usize       = 6;
pub const PPP_SESS_ID_SIZE:usize       = 2;
pub const PPP_MAX_SEG_SIZE_SIZE:usize  = 2;
pub const UNREACH_IP_SIZE:usize        = 4;
pub const UNREACH_PORT_SIZE:usize      = 2;
pub const PHY_CFG_SIZE:usize           = 1;
pub const VERSION_SIZE:usize           = 1;

pub struct ModeRegister(u8);
impl From<ModeRegister> for u8{
    fn from(mr: ModeRegister) -> Self {
        mr.0
    }
}
impl ModeRegister{
    // Reserved                 (0b01000101)
    pub const RST:   Self = Self(0b10000000);
    pub const WOL:   Self = Self(0b00100000);
    pub const PB:    Self = Self(0b00001000);
    pub const PPPOE: Self = Self(0b00001000);
    pub const FARP:  Self = Self(0b00000010);
}
impl Default for ModeRegister{
    fn default() -> Self {
        Self(0)
    }
}

pub struct PhyCfgRegister{
    reg: u8
}
impl From<PhyCfgRegister> for u8{
    fn from(phy: PhyCfgRegister) -> Self {
        phy.reg
    }
}
impl From<u8> for PhyCfgRegister{
    fn from(reg: u8) -> Self {
        Self{reg}
    }
}
impl PhyCfgRegister{
    pub fn enable_reset(mut self) -> Self{
        self.reg &= 0b00000000;
        self.reg |= 0b10000000;
        self
    }
    pub fn set_hw_opmd(mut self) -> Self{
        self.reg &= 0b10111111;
        self.reg |= 0b00000000;
        self
    }
    pub fn set_software_opmd(mut self) -> Self{
        self.reg &= 0b10111111;
        self.reg |= 0b01000000;
        self
    }
    pub fn set_10bt_hd(mut self) -> Self{
        self.reg &= 0b11000111;
        self.reg |= 0b00000000;
        self
    }
    pub fn set_10bt_fd(mut self) -> Self{
        self.reg &= 0b11000111;
        self.reg |= 0b00001000;
        self
    }
    pub fn set_100bt_hd(mut self) -> Self{
        self.reg &= 0b11000111;
        self.reg |= 0b00010000;
        self
    }
    pub fn set_100bt_fd(mut self) -> Self{
        self.reg &= 0b11000111;
        self.reg |= 0b00011000;
        self
    }
    pub fn set_100bt_fd_full(mut self) -> Self{
        self.reg &= 0b11000111;
        self.reg |= 0b00100000;
        self
    }
    pub fn set_not_used(mut self) -> Self{
        self.reg &= 0b11000111;
        self.reg |= 0b00101000;
        self
    }
    pub fn set_pwr_down(mut self) -> Self{
        self.reg &= 0b11000111;
        self.reg |= 0b00110000;
        self
    }
    pub fn set_all_capable(mut self) -> Self{
        self.reg &= 0b11000111;
        self.reg |= 0b00111000;
        self
    }
    pub fn is_full_duplex(&self) -> bool{
        self.reg & 0b00000100 == 0b00000100
    }
    pub fn is_half_duplex(&self) -> bool{
        if !self.is_full_duplex(){
            return true
        }
        false
    }
    pub fn is_100mpbs(&self) -> bool{
        self.reg & 0b00000010 == 0b00000010
    }
    pub fn is_10mbps(&self) -> bool{
        if !self.is_100mpbs(){
            return true
        }
        false
    }
    pub fn link_status(&self) -> bool {
        self.reg & 0b00000001 == 0b00000001
    }
}
impl Default for PhyCfgRegister{
    fn default() -> Self {
        Self{
            reg: 0b10000000,
        }
    }
}

pub struct CommonBlock{}
    
impl W5500{
    pub fn common_register() -> CommonBlock {
        CommonBlock{}
    }
}

impl CommonBlock{
    pub fn read_version_register(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u8{
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::VERSION, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<VERSION_SIZE>(header, spi, cs)[0]
    }
    pub fn read_mode_register(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u8 {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::MODE, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<MODE_SIZE>(header, spi, cs)[0]
    }
    pub fn write_mode_register(&self, mode: ModeRegister, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::MODE, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let mode = [mode.into()];
        write(header, &mode, spi, cs);
    }
    pub fn read_gateway_addr(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> [u8; GATEWAY_SIZE] {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::GATEWAY, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<GATEWAY_SIZE>(header, spi, cs)
    }
    pub fn write_gateway_addr(&self, gateway: impl Into<[u8;GATEWAY_SIZE]>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::GATEWAY, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let gateway = gateway.into();
        write(header, &gateway, spi, cs);
    }
    pub fn read_subnet(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> [u8; SUBNET_MASK_SIZE] {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::SUBNET_MASK, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<SUBNET_MASK_SIZE>(header, spi, cs)
    }
    pub fn write_subnet(&self, subnet: impl Into<[u8;SUBNET_MASK_SIZE]>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::SUBNET_MASK, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let subnet = subnet.into();
        write(header, &subnet, spi, cs);
    }
    pub fn read_mac(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> [u8; SOURCE_MAC_SIZE] {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::SOURCE_MAC, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<SOURCE_MAC_SIZE>(header, spi, cs)
    }
    pub fn write_mac(&self, mac: impl Into<[u8;SOURCE_MAC_SIZE]>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::SOURCE_MAC, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let mac = mac.into();
        write(header, &mac, spi, cs);
    }
    pub fn read_ip(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> [u8; SOURCE_IP_SIZE] {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::SOURCE_IP, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<SOURCE_IP_SIZE>(header, spi, cs)
    }
    pub fn write_ip(&self, ip: impl Into<[u8;SOURCE_IP_SIZE]>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::SOURCE_IP, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let ip = ip.into();
        write(header, &ip, spi, cs);
    }
    pub fn read_retry_time(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u16 {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::RETRY_TIME, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        u16::from_be_bytes(read::<RETRY_TIME_SIZE>(header, spi, cs))
    }
    pub fn write_retry_time(&self, retry_time: impl Into<[u8;RETRY_TIME_SIZE]>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::RETRY_TIME, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let rtr = retry_time.into();
        write(header, &rtr, spi, cs);
    }
    pub fn read_retry_count(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u8 {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::RETRY_COUNT, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<RETRY_COUNT_SIZE>(header, spi, cs)[0]
    }
    pub fn write_retry_count(&self, retry_count: impl Into<u8>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::RETRY_COUNT, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let rcr = [retry_count.into()];
        write(header, &rcr, spi, cs);
    }
    pub fn read_phy_cfg(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> PhyCfgRegister{
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::PHY_CFG, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        PhyCfgRegister::from(read::<PHY_CFG_SIZE>(header, spi, cs)[0])
    }
    pub fn write_phy_cfg(&self, phy_cfg: PhyCfgRegister, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        arduino_hal::delay_us(1);
        let header = header(CommonAddress::RETRY_COUNT, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let guard:u8 = 0b11111000;
        let phy:u8 = phy_cfg.into();
        let phy = [phy & guard];
        write(header, &phy, spi, cs);
    }
    
}
