#[derive(Clone, Copy)]
pub struct Bsb(u8);
impl From<Bsb> for u8{
    fn from(bsb: Bsb) -> Self {
        bsb.0
    }
}
impl Bsb{
 pub const COMMON:     Self =  Self(0b00000000);
 pub const SOCKET0:    Self =  Self(0b00001000); 
 pub const SOCKET0_TX :Self =  Self(0b00010000);  
 pub const SOCKET0_RX :Self =  Self(0b00011000);  
 pub const SOCKET1:    Self =  Self(0b00101000); 
 pub const SOCKET1_TX :Self =  Self(0b00110000);  
 pub const SOCKET1_RX :Self =  Self(0b00111000);  
 pub const SOCKET2:    Self =  Self(0b01001000); 
 pub const SOCKET2_TX :Self =  Self(0b01010000);  
 pub const SOCKET2_RX :Self =  Self(0b01011000);  
 pub const SOCKET3:    Self =  Self(0b01101000); 
 pub const SOCKET3_TX :Self =  Self(0b01110000);  
 pub const SOCKET3_RX :Self =  Self(0b01111000);  
 pub const SOCKET4:    Self =  Self(0b10001000); 
 pub const SOCKET4_TX :Self =  Self(0b10010000);  
 pub const SOCKET4_RX :Self =  Self(0b10011000);  
 pub const SOCKET5:    Self =  Self(0b10101000); 
 pub const SOCKET5_TX :Self =  Self(0b10110000);  
 pub const SOCKET5_RX :Self =  Self(0b10111000);  
 pub const SOCKET6:    Self =  Self(0b11001000); 
 pub const SOCKET6_TX :Self =  Self(0b11010000);  
 pub const SOCKET6_RX :Self =  Self(0b11011000);  
 pub const SOCKET7:    Self =  Self(0b11101000); 
 pub const SOCKET7_TX :Self =  Self(0b11110000);  
 pub const SOCKET7_RX :Self =  Self(0b11111000);
}

pub struct Rw(u8);
impl From<Rw> for u8{
    fn from(rw: Rw) -> Self {
        rw.0
    }
}
impl Rw{
    pub const READ:  Self = Self(0b00000000);
    pub const WRITE: Self = Self(0b00000100);
}

pub struct Om(u8);
impl From<Om> for u8{
    fn from(om: Om) -> Self {
        om.0
    }
}
impl Om{
    pub const VDM:  Self = Self(0b00000000);
    pub const FDL1: Self = Self(0b00000001);
    pub const FDL2: Self = Self(0b00000010);
    pub const FDL3: Self = Self(0b00000011);
}

pub struct ControlByte(u8);
impl From<u8> for ControlByte{
    fn from(data: u8) -> Self {
        Self(data)
    }
}
impl ControlByte{
    pub fn new(bsb: Bsb, rw: Rw, om: Om) -> ControlByte {
        let bsb:u8 = bsb.into();
        let rw:u8 = rw.into();
        let om:u8 = om.into();
        Self(bsb | rw | om)
    }
}
impl From<ControlByte> for u8{
    fn from(cb: ControlByte) -> Self {
        cb.0
    }
}



