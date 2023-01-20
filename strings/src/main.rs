#![no_std]
#![no_main]

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let data:u8 = 0x67;
    let phrase = "I luv you!";
    let bytes = phrase.as_bytes();
    // let bytes = [data];
    let pins = arduino_hal::pins!(dp);


    let mut led = pins.d13.into_output();
    let mut tgt = pins.d12.into_output();
    led.set_high();

    for byte in bytes.iter(){
        for bit_index in 0..u8::BITS{
            let mask:u8 = 0x80 >> bit_index;
            if *byte & mask == mask{
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
