use arduino_hal::pac::TC1;

use crate::timer::{Timer1, Waveform, ComMode, Clock};

const COUNTER_VALUE: u16 = 5000;

pub struct Servo{
    timer: Timer1,
}

impl Servo{
    pub fn new(use_pb1: bool, use_pb2: bool, timer: TC1) -> Servo {
        let timer = Timer1::new(timer, Waveform::FOURTEEN);
        // Set our pwm freqency to 50hz
        timer.load_icr1(COUNTER_VALUE);
        // Set out pin outputs
        if use_pb1{
            timer.set_coma(ComMode::FASTPWMNONINV);
            // Set out duty cycles to 50% (1.5ms)
            timer.load_ocr1a(23);
        }
        if use_pb2{
            timer.set_comb(ComMode::FASTPWMNONINV);
            // Set out duty cycles to 50% (1.5ms)
            timer.load_ocr1b(23);
        }
        // Start timer
        timer.set_clock(Clock::PRESCALE64);
        
        Self{
            timer,
        }
    }
    /// Angle should be given -90 - 90
    pub fn set_pb1_angle(&self, angle: f32){
        let factor:f32 = (angle + 90.0)/180.0;
        let duty = (5.0 * factor) + 5.0;
        let value = COUNTER_VALUE as f32 * (duty/100.0);

        self.timer.load_ocr1a(value as u16);
        
    }
    /// Angle should be given -90 - 90
    pub fn set_pb2_angle(&self, angle: f32){
        let factor:f32 = (angle + 90.0)/180.0;
        let duty = (5.0 * factor) + 5.0;
        let value = COUNTER_VALUE as f32 * (duty/100.0);

        self.timer.load_ocr1b(value as u16);
        
    }
    pub fn dissolve(self) -> Timer1 {
        self.timer.stop();
        self.timer
    }
}

