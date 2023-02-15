use arduino_hal::pac::TC1;

use crate::timer::{Timer1, Waveform, ComMode, Clock};

pub struct Servo{
    timer: Timer1,
}

impl Servo{
    pub fn new(use_pb1: bool, use_pb2: bool, timer: TC1) -> Servo {
        let timer = Timer1::new(timer, Waveform::FOURTEEN);
        // Set our pwm freqency to 50hz
        timer.load_icr1(312);
        // Set out duty cycles to 50% (1.5ms)
        timer.load_ocr1a(23);
        // Set out duty cycles to 50% (1.5ms)
        timer.load_ocr1b(23);
        // Set out pin outputs
        if use_pb1{
            timer.set_coma(ComMode::FASTPWMNONINV);
        }
        if use_pb2{
            timer.set_comb(ComMode::FASTPWMNONINV);
        }
        // Start timer
        timer.set_clock(Clock::PRESCALE1024);
        
        Self{
            timer,
        }
    }
    pub fn dissolve(self) -> Timer1 {
        self.timer.stop();
        self.timer
    }
}

