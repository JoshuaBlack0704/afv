#![no_std]
#![no_main]

use common_core::bits::Bits;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let phrase = "Hello World!";
    let bytes = phrase.as_bytes();
    let bytes = bytes.iter().map(|b| Bits::new(b));
    let pins = arduino_hal::pins!(dp);


    let mut led = pins.d13.into_output();
    let mut tgt = pins.d12.into_output();
    led.set_high();

    for bit in bytes{
        for bit in bit.boolean_iter(){
           if *bit{
                tgt.set_high();
                
            } 
            else{
                tgt.set_low();
            }
            arduino_hal::delay_ms(1);
        }
    }
    tgt.set_low();
    loop {
    }
}
