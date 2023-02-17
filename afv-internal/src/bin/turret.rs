#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use core::mem;

use afv_internal::servo::Servo;
use arduino_hal::{Spi, prelude::{_embedded_hal_blocking_spi_Write, _embedded_hal_spi_FullDuplex}, port::{Pin, mode::Output}};
use embedded_hal::{spi::{Polarity, Phase}, digital::v2::OutputPin};
use panic_halt as _;

struct InterruptState {
    blinker: Pin<Output>,
}


static mut INTERRUPT_STATE: mem::MaybeUninit<InterruptState> = mem::MaybeUninit::uninit();


#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);
    let mut serial = arduino_hal::default_serial!(peripherals, pins, 57600);    
    let mut state = pins.d2.into_output();
    state.set_low();


    // Is interrupted on COMPB
    let servo2 = pins.d8.into_output();
    


    

    unsafe {
        // SAFETY: Interrupts are not enabled at this point so we can safely write the global
        // variable here.  A memory barrier afterwards ensures the compiler won't reorder this
        // after any operation that enables interrupts.
        INTERRUPT_STATE = mem::MaybeUninit::new(InterruptState {
            blinker: servo2.downgrade(),
        });
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }    
    // let mut w5500 = W5500::new(spi, cs);
    // if let Ok(_) = w5500.version(){
    //     state.set_high(); 
    // }
    // let gateway = Ipv4Addr::new(169,254,182,0);
    // let _ = w5500.set_gar(&gateway);
    // let subnet = Ipv4Addr::new(255,255,255,0);
    // let _ = w5500.set_subr(&subnet);
    // let _ = w5500.set_shar(&w5500_hl::net::Eui48Addr { octets: [0x00,0x08,0xDC,0x01,0x02,0x03] });
    // let ip = Ipv4Addr::new(169,254,182,99);
    // let _ = w5500.set_sipr(&ip);
    // let rcr = 0x0007;
    // let _ = w5500.set_rcr(rcr);
    // let mut cfg = PhyCfg::default();
    // cfg = cfg.set_opmdc(w5500_hl::ll::OperationMode::Auto);
    // let _ = w5500.set_phycfgr(cfg);

    // let _ = w5500.set_sn_port(w5500_hl::ll::Sn::Sn0, 1000);
    // let _ = w5500.set_sn_mr(w5500_hl::ll::Sn::Sn0, SocketMode::default().set_protocol(w5500_hl::ll::Protocol::Tcp));
    // let _ = w5500.set_sn_cr(w5500_hl::ll::Sn::Sn0, w5500_hl::ll::SocketCommand::Listen);
    
    // Enable interrupts globally, not a replacement for the specific interrupt enable
    unsafe {
        // SAFETY: Not inside a critical section and any non-atomic operations have been completed
        // at this point.
        avr_device::interrupt::enable();
    }    

    
    let mut cs = pins.d10.into_output();
    cs.set_high();
    let mut mosi = pins.d11.into_output();
    mosi.set_high();
    let miso = pins.d12.into_pull_up_input();
    let sck = pins.d13.into_output();
    let mut settings = arduino_hal::spi::Settings::default();
    settings.clock = arduino_hal::spi::SerialClockRate::OscfOver128;
    settings.mode.polarity = Polarity::IdleLow;
    settings.mode.phase = Phase::CaptureOnFirstTransition; 
    let (mut spi, mut cs) = Spi::new(peripherals.SPI, sck, mosi, miso, cs, settings);

    let _ = cs.set_low();
    let addr = 0x0039u16.to_be_bytes();
    let control = 0u8.to_be_bytes();
    let _ = spi.write(&addr);
    let _ = spi.write(&control);
    let _ = spi.write(&[0]);
    let v = spi.read();
    if let Ok(_v)=v{
        ufmt::uwriteln!(&mut serial, "Version: {}", _v);
        if _v == 0x04{
            state.set_high();
        }
    }
    let _ = cs.set_high();


    arduino_hal::delay_ms(500);
    let _ = cs.set_low();
    let _ = ufmt::uwriteln!(&mut serial, "Starting ip transfer");
    let addr = 0x0001u16.to_be_bytes();
    let control = 0b00000100u8.to_be_bytes();
    let _ = spi.write(&addr);
    let _ = spi.write(&control);
    let ip_addr: [u8;4] = [192,168,20,0];
    let _ = spi.write(&ip_addr);

    let _ = cs.set_high();
    
    arduino_hal::delay_ms(500);
    let _ = cs.set_low();
    let _ = ufmt::uwriteln!(&mut serial, "Starting ip transfer");
    let addr = 0x0001u16.to_be_bytes();
    let control = 0b00000000u8.to_be_bytes();
    let _ = spi.write(&addr);
    let _ = spi.write(&control);
    let mut ip_addr: [u8;4] = [0,0,0,0];
    let _ = spi.write(&[0]);
    ip_addr[0] = spi.read().unwrap();
    let _ = spi.write(&[0]);
    ip_addr[1] = spi.read().unwrap();
    let _ = spi.write(&[0]);
    ip_addr[2] = spi.read().unwrap();
    let _ = spi.write(&[0]);
    ip_addr[3] = spi.read().unwrap();

    let _ = cs.set_high();

    
    let _ = ufmt::uwriteln!(&mut serial, "Returned Ip addr: {:?}", ip_addr);

    let servo_ctl = Servo::new(
        true, 
        false, 
        false,
        true,
        true,
        peripherals.TC1);
    // Is hooked to PB1
    let _servo1 = pins.d9.into_output();

    loop {
        servo_ctl.set_pb1_angle(0.0);
        servo_ctl.set_pb2_angle(90.0);
        arduino_hal::delay_ms(1000);
        servo_ctl.set_pb1_angle(0.0);
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
