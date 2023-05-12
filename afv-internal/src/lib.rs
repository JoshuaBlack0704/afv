//! Before attempting to learn how to use this codebase it is GREATLY recommended
//! to learn [Rust](https://www.rust-lang.org).
//! Once you have installed rust throug [Rustup](https://rustup.rs) you can use `cargo doc --open` to
//! view this documention
//!
//! It is highly recommended to familiarize yourself with the following crates:
//! * [arduino_hal](https://rahix.github.io/avr-hal/arduino_hal/index.html)
//! * [embedded_hal](https://docs.rs/embedded_hal/latest/embedded_hal)
//! * [Postcard](https://docs.rs/postcard/latest/postcard)
//!
//! It is also highly recommended to get familar with the [Rust](https://rust-lang.org) [embedded ecosystem](https://www.rust-lang.org/what/embedded).
//!
//! This crate is based off of the [Arduino Hal](https://github.com/Rahix/avr-hal) base framework
//! that provides the correct linker scripts, avrdude integration for USB programming on an Arduino, and ravedude front end for use
//! with `Cargo run` and `Cargo build`
//!
//! NOTE: This crate and its containd binary programs shoud ONLY EVER BE RUN IN RELEASE MODE.

#![no_std]

/// The port of the FLIR turret TCP server
pub const FLIR_TURRET_PORT: u16 = 3031;
/// The port of the Nozzle turret TCP server
pub const NOZZLE_TURRET_PORT: u16 = 3032;
/// The port of the Lidar TCP server
pub const LIDAR_PORT: u16 = 3033;
/// The port of the Pump TCP server
pub const PUMP_PORT: u16 = 3034;
/// The port of the Lights TCP server
pub const LIGHTS_PORT: u16 = 3035;
/// The port of the Sirens TCP server
pub const SIREN_PORT: u16 = 3036;
/// The packet size of communcation between the GCS-AFV drivers and the AFV-INTERNAL drivers
pub const SOCKET_MSG_SIZE: usize = 256;
pub const PAN_STEPPER_STEPS_REV: u32 = 200;
pub const TILT_STEPPER_STEPS_REV: u32 = 200;

/// Provides the networking primatives to send data over the Ethernet shields
pub mod network;

/// This module contains the driver firmware for the AFV's onboard [Garmin Lidar](https://garmin.com/en-US/p/557294)
pub mod garmin_lidar_v3;
/// This module contains the driver firmware for operating the Wiznet [W5500](https://wiznet.io/product-item/w5500) chip that is on the
/// the [Arduino Ethernet Shield 2](https://store-usa.arduino.cc/products/arduino-ethernet-shield-2?selectedStore=us) boards
pub mod w5500;

/// This module contains stepper acutation firmware for use with the AFV's turret control PCBs which
/// have the [Texas Instruments DRV8886AT](https://ti.com/product/DRV8886AT) onboard.
pub mod stepper;

/// This module contains a wrapper class to control two stepper motors in a pan-tilt configuration through a TCP server
pub mod turret;

/// This module provides a convenience wrapper around the 16 bit timer in the [Arduino Uno R3's](https://store-usa.arduino.cc/products/arduino-uno-rev3?selectedStore=us) [Atmega328p](https://www.microchip.com/en-us/product/ATmega328P) chip
pub mod timer;

/// This module provides a convenience wrapper for controlling a servo using the [timer] module
pub mod servo;

/// This module provides a convenience wrapper for controlling the [garmin_lidar_v3] driver and transeiving data on 
/// a TCP server
pub mod lidar;

/// This module provides a convenience wrapper for controlling the pump and transeiving data on 
/// a TCP server
pub mod pump;

/// This module provides a convenience wrapper for controlling the lights and transeiving data on 
/// a TCP server
pub mod lights;

/// This module provides a convenience wrapper for controlling the sirens and transeiving data on 
/// a TCP server
pub mod sirens;
