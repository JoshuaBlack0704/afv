#![no_std]

pub const MAINCTLPORT: u16 = 3030;
pub const FLIR_TURRET_PORT: u16 = 3031;
pub const NOZZLE_TURRET_PORT: u16 = 3032;
pub const LIDAR_PORT: u16 = 3033;
pub const SOCKET_MSG_SIZE: usize = 256;
pub const PAN_STEPPER_STEPS_REV: u32 = 200;
pub const TILT_STEPPER_STEPS_REV: u32 = 200;

pub mod network;

pub mod w5500;

pub mod stepper;

pub mod turret;

pub mod timer;

pub mod servo;

pub mod mainctl;

pub mod lidar;