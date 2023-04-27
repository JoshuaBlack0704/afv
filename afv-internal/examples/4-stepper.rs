 #![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]


struct States(bool, bool, bool, bool);
impl States{
    const ZERO: Self = Self(true, false, false, false);
    const ONE: Self = Self(true, false, true, false);
    const TWO: Self = Self(false, false, true, false);
    const THREE: Self = Self(false, true, true, false);
    const FOUR: Self = Self(false, true, false, false);
    const FIVE: Self = Self(false, true, false, true);
    const SIX: Self = Self(false, false, false, true);
    const SEVEN: Self = Self(true, false, false, true);

    const STEPS: [Self; 8] = [Self::ZERO,Self::ONE,Self::TWO,Self::THREE,Self::FOUR,Self::FIVE,Self::SIX,Self::SEVEN];
}

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);
    let mut serial = arduino_hal::default_serial!(peripherals, pins, 57600);    
    let _ = ufmt::uwriteln!(&mut serial, "On");

    let mut state1 = true;
    let mut state2 = false;


    let mut pin2 = pins.d2.into_output();
    let mut pin3 = pins.d3.into_output_high();
    let mut pin4 = pins.d4.into_output();
    let mut pin5 = pins.d5.into_output_high();
    
    loop {
        for step in States::STEPS{
            arduino_hal::delay_ms(1);
            if step.0 {pin2.set_high()}
            else {pin2.set_low()}
            if step.1 {pin3.set_high()}
            else {pin3.set_low()}
            if step.2 {pin4.set_high()}
            else {pin4.set_low()}
            if step.3 {pin5.set_high()}
            else {pin5.set_low()}
        }
    }
}
