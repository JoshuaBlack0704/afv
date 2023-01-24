use super::W5500Builder;

impl W5500Builder{
    pub fn new() -> W5500Builder {
        Self{
            wol: todo!(),
            ping_block: todo!(),
            pppoe: todo!(),
            farp: todo!(),
            gateway: todo!(),
            subnet: todo!(),
            mac: todo!(),
            ip: todo!(),
        }
    }
    /// Will set bit 5 of Mode Register in Common Registers
    /// BSB: 00000 Offset: 0x0000
    pub fn set_wake_on_lan(mut self, option: bool) -> W5500Builder {
        self.wol = option;
        self
    }
    pub fn set_ping_block(mut self, option: bool) -> W5500Builder {
        self.ping_block = option;
        self
    }
    pub fn set_pppoe(mut self, option: bool) -> W5500Builder {
        self.pppoe= option;
        self
    }
    pub fn set_farp(mut self, option: bool) -> W5500Builder {
        self.farp = option;
        self
    }
    pub fn set_gateway(mut self, gateway: [u8;4]) -> W5500Builder {
        self.gateway = gateway;
        self
    }
    pub fn set_subnet(mut self, subnet: [u8;4]) -> W5500Builder {
        self.subnet = subnet;
        self
    }
    pub fn set_mac(mut self, mac: [u8;6]) -> W5500Builder {
        self.mac = mac;
        self
    }
    pub fn set_ip(mut self, ip: [u8;4]) -> W5500Builder {
        self.ip = ip;
        self
    }
    pub fn mode(&self){
        let mut mode:u8 = 0;
        // wol
        let mut mask = 0;
        if self.wol{
            mask = 0x80;
        }
        mask >>= 2;
        mode ^= mask;
        
        // PB
        let mut mask = 0;
        if self.ping_block{
            mask = 0x80;
        }
        mask >>= 3;
        mode ^= mask;
        
        // pppoe
        let mut mask = 0;
        if self.pppoe{
            mask = 0x80;
        }
        mask >>= 4;
        mode ^= mask;
        
        // farp
        let mut mask = 0;
        if self.farp{
            mask = 0x80;
        }
        mask >>= 6;
        mode ^= mask;
    }
}

