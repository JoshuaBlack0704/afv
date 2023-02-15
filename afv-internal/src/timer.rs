use arduino_hal::pac::TC1;

#[derive(PartialEq, Eq)]
pub struct Waveform(u8,u8);
impl Waveform{
    pub const FOURTEEN: Self = Self(0b11, 0b10);
}
pub struct ComMode(u8);
impl ComMode{
    pub const DISCONNECTED: Self = Self(0b00);
    pub const FASTPWMNONINV: Self = Self(0b10);
}
pub struct Clock(u8);
impl Clock{
    pub const PRESCALE1024: Self = Self(0b101);
}

pub struct Timer1{
    wgw: Waveform,
    timer: TC1,
}

impl Timer1{

    pub fn new(timer: TC1, wgw: Waveform) -> Timer1 {
        // Ensure timer is stopped
        timer.tccr1b.write(|w| w.cs1().no_clock());
        // Clear timer value
        timer.tcnt1.write(|w| w.bits(0));
        // Set wave form
        timer.tccr1a.write(|w| w.wgm1().bits(wgw.1));
        timer.tccr1b.write(|w| w.wgm1().bits(wgw.0));
        
        Timer1{
            timer,
            wgw,
        }
        
    }

    pub fn set_coma(&self, com: ComMode){
        self.timer.tccr1a.write(|w| w.com1a().bits(com.0));
    }
    pub fn set_comb(&self, com: ComMode){
        self.timer.tccr1a.write(|w| w.com1a().bits(com.0));
    }
    pub fn disconnect_coma(&self){
        self.timer.tccr1a.write(|w| w.com1a().disconnected());
    }
    pub fn disconnect_comb(&self){
        self.timer.tccr1a.write(|w| w.com1a().disconnected());
    }
    pub fn load_icr1(&self, icr1: u16){
        if self.wgw == Waveform::FOURTEEN{
            self.timer.icr1.write(|w| w.bits(icr1));
        }
    }
    pub fn load_ocr1a(&self, ocr1a: u16){
        self.timer.ocr1a.write(|w| w.bits(ocr1a));
    }
    pub fn load_ocr1b(&self, ocr1b: u16){
        self.timer.ocr1b.write(|w| w.bits(ocr1b));
    }
    pub fn set_clock(&self, clock: Clock){
        self.timer.tccr1b.write(|w| w.cs1().bits(clock.0));
    }
    pub fn stop(&self){
        self.timer.tccr1b.write(|w| w.cs1().no_clock());
    }
    pub fn dissolve(self) -> TC1 {
        self.stop();
        self.timer
    }
}

