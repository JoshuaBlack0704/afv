#![no_std]

pub const TESTPORT: u16 = 3030;
pub const SOCKET_MSG_SIZE: usize = 10;

pub mod network;

pub mod w5500;

pub mod timer;

pub mod servo;

pub mod mainctl;