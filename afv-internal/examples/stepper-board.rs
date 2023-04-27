 #![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]



use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);
    let mut serial = arduino_hal::default_serial!(peripherals, pins, 57600);    
    let _ = ufmt::uwriteln!(&mut serial, "On");
    let mut a5 = pins.a5.into_output();
    let mut a4 = pins.a4.into_output();
    let mut a3 = pins.a3.into_output();

    loop{
        a5.set_high();
        a4.set_high();
        a3.set_high();
        arduino_hal::delay_ms(1000);
        a5.set_low();
        a4.set_low();
        a3.set_low();
        arduino_hal::delay_ms(1000);
        
    }

    // let mut pin2 = pins.d2.into_output();
    // pin2.set_low();
    // let mut pin3 = pins.d3.into_output();
    // let mut pin4 = pins.d4.into_output();
    // pin4.set_low();
    // let mut pin5 = pins.d5.into_output();
    // let pin6 = pins.d6.into_floating_input();
    // loop {
    //     if pin6.is_high(){
    //         // arduino_hal::delay_us(50);
    //         // pin3.set_high();
    //         // pin5.set_high();
    //         // arduino_hal::delay_us(50);
    //         // pin3.set_low();
    //         // pin5.set_low();
    //     }
    //     for _ in 0..16{
    //         pin3.set_high();
    //         pin5.set_high();
    //         arduino_hal::delay_us(20);
    //         pin3.set_low();
    //         pin5.set_low();
    //         arduino_hal::delay_us(20);
    //     } 
    //     arduino_hal::delay_ms(1);
    // }
}
