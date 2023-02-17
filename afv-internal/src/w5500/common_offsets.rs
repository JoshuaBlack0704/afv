pub struct CommonAddress(u16);
impl From<CommonAddress> for u16{
    fn from(ca: CommonAddress) -> Self {
        ca.0
    }
}

impl CommonAddress{
    pub const MODE:              Self = Self(0x0000);
    pub const GATE_WAY:          Self = Self(0x0001);
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
