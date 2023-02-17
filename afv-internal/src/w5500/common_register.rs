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

pub struct PhyCfgRegister(u8);
impl From<PhyCfgRegister> for u8{
    fn from(phy: PhyCfgRegister) -> Self {
        phy.0
    }
}
impl PhyCfgRegister{
    /// To reset use &
    pub const RST:         Self = Self(0b0_0_000_000);
    pub const OPMD_OPMDC:  Self = Self(0b1_1_000_000);
    pub const OPMD_HW:     Self = Self(0b1_0_000_000);
    pub const BT10_HD:     Self = Self(0b1_0_000_000);
    pub const BT10_FD:     Self = Self(0b1_0_001_000);
    pub const BT100_HD:    Self = Self(0b1_0_010_000);
    pub const BT100_FD:    Self = Self(0b1_0_011_000);
    pub const BT100_HD_AN: Self = Self(0b1_0_100_000);
    pub const NOT_UNSED:   Self = Self(0b1_0_101_000);
    pub const PWR_DOWN:    Self = Self(0b1_0_110_000);
    pub const ALL_AN:      Self = Self(0b1_0_111_000);
    pub const FULL_DUPLEX: Self = Self(0b1_0_000_100);
    pub const HALF_DUPLEX: Self = Self(0b1_0_000_000);
    pub const HIGH_SPEED:  Self = Self(0b1_0_000_010);
    pub const LOW_SPEED:   Self = Self(0b1_0_000_000);
    pub const LINK_UP:     Self = Self(0b1_0_000_001);
    pub const LINK_DOWN:   Self = Self(0b1_0_000_000);
}

pub struct CommonBlock{}
    
impl W5500{
    pub fn common_register() -> CommonBlock {
        CommonBlock{}
    }
}

impl CommonBlock{
    pub fn read_version_register(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u8{
        let header = header(CommonAddress::VERSION, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<VERSION_SIZE>(header, spi, cs)[0]
    }
    pub fn read_mode_register(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u8 {
        let header = header(CommonAddress::MODE, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<MODE_SIZE>(header, spi, cs)[0]
    }
    pub fn write_mode_register(&self, mode: ModeRegister, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        let header = header(CommonAddress::MODE, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let mode = [mode.into()];
        write(header, &mode, spi, cs);
    }
    pub fn read_gateway_addr(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> [u8; GATEWAY_SIZE] {
        let header = header(CommonAddress::GATEWAY, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<GATEWAY_SIZE>(header, spi, cs)
    }
    pub fn write_gateway_addr(&self, gateway: impl Into<[u8;GATEWAY_SIZE]>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        let header = header(CommonAddress::GATEWAY, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let gateway = gateway.into();
        write(header, &gateway, spi, cs);
    }
    pub fn read_subnet(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> [u8; SUBNET_MASK_SIZE] {
        let header = header(CommonAddress::SUBNET_MASK, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<SUBNET_MASK_SIZE>(header, spi, cs)
    }
    pub fn write_subnet(&self, subnet: impl Into<[u8;SUBNET_MASK_SIZE]>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        let header = header(CommonAddress::SUBNET_MASK, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let subnet = subnet.into();
        write(header, &subnet, spi, cs);
    }
    pub fn read_mac(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> [u8; SOURCE_MAC_SIZE] {
        let header = header(CommonAddress::SOURCE_MAC, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<SOURCE_MAC_SIZE>(header, spi, cs)
    }
    pub fn write_mac(&self, mac: impl Into<[u8;SOURCE_MAC_SIZE]>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        let header = header(CommonAddress::SOURCE_MAC, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let mac = mac.into();
        write(header, &mac, spi, cs);
    }
    pub fn read_ip(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> [u8; SOURCE_IP_SIZE] {
        let header = header(CommonAddress::SOURCE_IP, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<SOURCE_IP_SIZE>(header, spi, cs)
    }
    pub fn write_ip(&self, ip: impl Into<[u8;SOURCE_IP_SIZE]>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        let header = header(CommonAddress::SOURCE_IP, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let ip = ip.into();
        write(header, &ip, spi, cs);
    }
    pub fn read_retry_time(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u16 {
        let header = header(CommonAddress::RETRY_TIME, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        u16::from_be_bytes(read::<RETRY_TIME_SIZE>(header, spi, cs))
    }
    pub fn write_retry_time(&self, retry_time: impl Into<[u8;RETRY_TIME_SIZE]>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        let header = header(CommonAddress::RETRY_TIME, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let rtr = retry_time.into();
        write(header, &rtr, spi, cs);
    }
    pub fn read_retry_count(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u8 {
        let header = header(CommonAddress::RETRY_COUNT, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<RETRY_COUNT_SIZE>(header, spi, cs)[0]
    }
    pub fn write_retry_count(&self, retry_count: impl Into<u8>, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        let header = header(CommonAddress::RETRY_COUNT, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let rcr = [retry_count.into()];
        write(header, &rcr, spi, cs);
    }
    pub fn read_phy_cfg(&self, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) -> u8{
        let header = header(CommonAddress::PHY_CFG, ControlByte::new(COMMON_BLOCK, Rw::READ, Om::VDM));
        read::<PHY_CFG_SIZE>(header, spi, cs)[0]
    }
    pub fn write_phy_cfg(&self, phy_cfg: PhyCfgRegister, spi: &mut Spi, cs: &mut ChipSelectPin<PB2>) {
        let header = header(CommonAddress::RETRY_COUNT, ControlByte::new(COMMON_BLOCK, Rw::WRITE, Om::VDM));
        let guard:u8 = 0b11111000;
        let phy:u8 = phy_cfg.into();
        let phy = [phy & guard];
        write(header, &phy, spi, cs);
    }
    
}
