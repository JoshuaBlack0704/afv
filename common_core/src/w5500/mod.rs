pub struct Common_Bsb(u8);
impl Common_Bsb{
    pub const COMMON:Self = Self(0x00);
}
pub struct Common_Registers(u16);
impl Common_Registers{
    pub const MR: Self = Self(0x0000);
    pub const GAR0: Self = Self(0x0001);
    pub const GAR1: Self = Self(0x0002);
    pub const GAR2: Self = Self(0x0003);
    pub const GAR3: Self = Self(0x0004);
    pub const SUBR0: Self = Self(0x0005);
    pub const SUBR1: Self = Self(0x0006);
    pub const SUBR2: Self = Self(0x0007);
    pub const SUBR3: Self = Self(0x0008);
    pub const SHAR0: Self = Self(0x0009);
    pub const SHAR1: Self = Self(0x000A);
    pub const SHAR2: Self = Self(0x000B);
    pub const SHAR3: Self = Self(0x000C);
    pub const SHAR4: Self = Self(0x000D);
    pub const SHAR5: Self = Self(0x000E);
    pub const SIPR0: Self = Self(0x000F);
    pub const SIPR1: Self = Self(0x0010);
    pub const SIPR2: Self = Self(0x0011);
    pub const SIPR3: Self = Self(0x0012);
    pub const INTLEVEL0: Self = Self(0x0013);
    pub const INTLEVEL1: Self = Self(0x0014);
    pub const IR: Self = Self(0x0015);
    pub const IMR: Self = Self(0x0016);
    pub const SIR: Self = Self(0x0017);
    pub const SIMR: Self = Self(0x0018);
    pub const RTR0: Self = Self(0x0019);
    pub const RTR1: Self = Self(0x001A);
    pub const RCR: Self = Self(0x001B);
    pub const PTIMER: Self = Self(0x001C);
    pub const PMAGIC: Self = Self(0x001D);
    pub const PHAR0: Self = Self(0x001E);
    pub const PHAR1: Self = Self(0x001F);
    pub const PHAR2: Self = Self(0x0020);
    pub const PHAR3: Self = Self(0x0021);
    pub const PHAR4: Self = Self(0x0022);
    pub const PHAR5: Self = Self(0x0023);
    pub const PSID0: Self = Self(0x0024);
    pub const PSID1: Self = Self(0x0025);
    pub const PMRU0: Self = Self(0x0026);
    pub const PMRU1: Self = Self(0x0027);
    pub const UIPR0: Self = Self(0x0028);
    pub const UIPR1: Self = Self(0x0029);
    pub const UIPR2: Self = Self(0x002A);
    pub const UIPR3: Self = Self(0x002B);
    pub const UPORT0: Self = Self(0x002C);
    pub const UPORT1: Self = Self(0x002D);
    pub const PHYCFGR: Self = Self(0x002E);
    pub const VERSIONR: Self = Self(0x0039);
}
pub struct Socket_Bsb(u8);
impl Socket_Bsb{
    pub const S0: Self = Self(0x08);
    pub const S0TX: Self = Self(0x10);
    pub const S0RX: Self = Self(0x18);
    
    pub const S1: Self = Self(0x28);
    pub const S1TX: Self = Self(0x30);
    pub const S1RX: Self = Self(0x38);
    
    pub const S2: Self = Self(0x48);
    pub const S2TX: Self = Self(0x50);
    pub const S2RX: Self = Self(0x58);
    
    pub const S3: Self = Self(0x68);
    pub const S3TX: Self = Self(0x70);
    pub const S3RX: Self = Self(0x78);
    
    pub const S4: Self = Self(0x88);
    pub const S4TX: Self = Self(0x90);
    pub const S4RX: Self = Self(0x98);
    
    pub const S5: Self = Self(0xA8);
    pub const S5TX: Self = Self(0xB0);
    pub const S5RX: Self = Self(0xB8);
    
    pub const S6: Self = Self(0xC8);
    pub const S6TX: Self = Self(0xD0);
    pub const S6RX: Self = Self(0xD8);
    
    pub const S7: Self = Self(0xE8);
    pub const S7TX: Self = Self(0xF0);
    pub const S7RX: Self = Self(0xF8);
}
pub struct Socket_Registers(u16);
impl Socket_Registers{
    pub const SN_MR: Self = Self(0x0000);
    pub const SN_CR: Self = Self(0x0001);
    pub const SN_IR: Self = Self(0x0002);
    pub const SN_SR: Self = Self(0x0003);
    pub const SN_PORT0: Self = Self(0x0004);
    pub const SN_PORT1: Self = Self(0x0005);
    pub const SN_DHAR0: Self = Self(0x0006);
    pub const SN_DHAR1: Self = Self(0x0007);
    pub const SN_DHAR2: Self = Self(0x0008);
    pub const SN_DHAR3: Self = Self(0x0009);
    pub const SN_DHAR4: Self = Self(0x000A);
    pub const SN_DHAR5: Self = Self(0x000B);
    pub const SN_DIPR0: Self = Self(0x000C);
    pub const SN_DIPR1: Self = Self(0x000D);
    pub const SN_DIPR2: Self = Self(0x000E);
    pub const SN_DIPR3: Self = Self(0x000F);
    pub const SN_DPORT0: Self = Self(0x0010);
    pub const SN_DPORT1: Self = Self(0x0011);
    pub const SN_MSSR0: Self = Self(0x0012);
    pub const SN_MSSR1: Self = Self(0x0013);
    pub const SN_TOS: Self = Self(0x0015);
    pub const SN_TTL: Self = Self(0x0016);
    pub const SN_RXBUF_SIZE: Self = Self(0x001E);
    pub const SN_TXBUF_SIZE: Self = Self(0x001F);
    pub const SN_TX_FSR0: Self = Self(0x0020);
    pub const SN_TX_FSR1: Self = Self(0x0021);
    pub const SN_TX_RD0: Self = Self(0x0022);
    pub const SN_TX_RD1: Self = Self(0x0023);
    pub const SN_TX_WR0: Self = Self(0x0024);
    pub const SN_TX_WR1: Self = Self(0x0025);
    pub const SN_RX_RSR0: Self = Self(0x0026);
    pub const SN_RX_RSR1: Self = Self(0x0027);
    pub const SN_RX_RD0: Self = Self(0x0028);
    pub const SN_RX_RD1: Self = Self(0x0029);
    pub const SN_RX_WR0: Self = Self(0x002A);
    pub const SN_RX_WR1: Self = Self(0x002B);
    pub const SN_IMR: Self = Self(0x002C);
    pub const SN_FRAG0: Self = Self(0x002D);
    pub const SN_FRAG1: Self = Self(0x002E);
    pub const SN_KPALVTR: Self = Self(0x002F);
}

pub mod builder;
pub struct W5500Builder{
    /// Wake On Lan
    wol: bool,
    /// Ping Block Mode
    ping_block: bool,
    /// Point to Point Protocol over Ethernet
    pppoe: bool,
    /// Force ARP
    farp: bool,
    /// Gateway ipv4 addr
    gateway: [u8;4],
    /// Subnet mask
    subnet: [u8;4],
    /// Hardware MAC addr
    mac: [u8;6],
    /// The source ip addr
    ip: [u8;4],
    
}
pub struct W5500{
    
}