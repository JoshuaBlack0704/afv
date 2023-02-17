pub struct SocketAddress(u16);
impl From<SocketAddress> for u16{
    fn from(sa: SocketAddress) -> Self {
        sa.0
    }
}
impl SocketAddress{
    pub const MODE: Self = Self(0x0000);
    pub const COMMAND: Self = Self(0x0001);
    pub const INTERRUPT: Self = Self(0x0002);
    pub const STATUS: Self = Self(0x0003);
    pub const SOURCE_PORT: Self = Self(0x0004);
    pub const DST_MAC_ADD: Self = Self(0x0006);
    pub const DST_IP: Self = Self(0x000c);
    pub const DST_PORT: Self = Self(0x0010);
    pub const MAX_SEG_SIZE: Self = Self(0x0012);
    pub const IP_TOS: Self = Self(0x0015);
    pub const IP_TTL: Self = Self(0x0016);
    pub const RX_BUFF_SIZE: Self = Self(0x001e);
    pub const TX_BUFF_SIZE: Self = Self(0x001f);
    pub const MODE: Self = Self(0x0000);
    pub const MODE: Self = Self(0x0000);
    pub const MODE: Self = Self(0x0000);
    pub const MODE: Self = Self(0x0000);
    pub const MODE: Self = Self(0x0000);
    pub const MODE: Self = Self(0x0000);
    pub const MODE: Self = Self(0x0000);
    pub const MODE: Self = Self(0x0000);
    pub const MODE: Self = Self(0x0000);
}