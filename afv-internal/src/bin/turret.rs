#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use core::mem;

use afv_internal::servo::Servo;
use arduino_hal::{Spi, spi::Settings, port::{Pin, mode::Output}};
use avr_device::interrupt;
use panic_halt as _;
use w5500_hl::ll::blocking::fdm::W5500;

struct InterruptState {
    blinker: Pin<Output>,
}


static mut INTERRUPT_STATE: mem::MaybeUninit<InterruptState> = mem::MaybeUninit::uninit();


#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);
    // let cs = pins.d10.into_output();
    // let mosi = pins.d11.into_output();
    // let miso = pins.d12.into_pull_up_input();
    // let sck = pins.d13.into_output();
    // let (spi, _cs) = Spi::nBew(peripherals.SPI, sck, mosi, miso, cs, Settings::default());

    let led = pins.d13.into_output();

    unsafe {
        // SAFETY: Interrupts are not enabled at this point so we can safely write the global
        // variable here.  A memory barrier afterwards ensures the compiler won't reorder this
        // after any operation that enables interrupts.
        INTERRUPT_STATE = mem::MaybeUninit::new(InterruptState {
            blinker: led.downgrade(),
        });
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }    

    


    
    let servo_ctl = Servo::new(
        true, 
        true, 
        false,
        true,
        true,
        peripherals.TC1);
    let _d9 = pins.d9.into_output();
    let _d10 = pins.d10.into_output();

    // let w5500 = W5500::new(spi);
    
    // Enable interrupts globally, not a replacement for the specific interrupt enable
    unsafe {
        // SAFETY: Not inside a critical section and any non-atomic operations have been completed
        // at this point.
        avr_device::interrupt::enable();
    }    

    loop {
        servo_ctl.set_pb1_angle(40.0);
        servo_ctl.set_pb2_angle(2500.0);
        arduino_hal::delay_ms(1000);
        servo_ctl.set_pb1_angle(-40.0);
        servo_ctl.set_pb2_angle(-90.0);
        arduino_hal::delay_ms(1000);
    }
}

#[avr_device::interrupt(atmega328p)]
fn TIMER1_COMPB(){
    let state = unsafe {
        // SAFETY: We _know_ that interrupts will only be enabled after the LED global was
        // initialized so this ISR will never run when LED is uninitialized.
        &mut *INTERRUPT_STATE.as_mut_ptr()
    };

    state.blinker.set_low();   
    
}
#[avr_device::interrupt(atmega328p)]
fn TIMER1_CAPT(){
    let state = unsafe {
        // SAFETY: We _know_ that interrupts will only be enabled after the LED global was
        // initialized so this ISR will never run when LED is uninitialized.
        &mut *INTERRUPT_STATE.as_mut_ptr()
    };

    state.blinker.set_high();   
    
}
