//! Before attempting to learn how to use this codebase it is GREATLY recommended
//! to learn [Rust](https://www.rust-lang.org).
//! Once you have installed rust through [Rustup](https://rustup.rs) you can use `cargo doc --open` from within the gcs-afv folder to
//! view its documention. Same goes for afv-internal
//!
//! It is highly recommended to familiarize yourself with the following crates:
//! * [Eframe](https://docs.rs/eframe/latest/eframe)
//! * [Retina](https://docs.rs/retina/latest/retina)
//! * [Tokio](https://docs.rs/tokio/latest/tokio)
//! * [Image](https://docs.rs/image/latest/image)
//!
//! If you have any questions please feel free to reach me (Joshua Black) at:
//! * `Text`:  +1 417-848-8609
//! * `Email`: jtblack0704@gmail.com
//!
//! At the end of your ownership of this codebase, please either delete my contact info or replace it with your own.
//!
//! GCS-AFV are the main control systems that can run on Linux or Windows
//!
//! This crate provides the faculties neccessary to establish communicaton and control over
//! all of the elements of the AFV.
//!
//! NOTE: Both the Control Station's code and AFV's code are located inside this crate
//!
//! The concept behind this system is to use a single bus to distribute data to EVERY connected process
//! regardless of if that process is running on the Control Station or the AFV.
//! This is acheived by using the [network::afv_bridge::AfvBridge] struct to provide transperent forwarding of messages across the
//! the network.
//! The [tokio::sync::broadcast] struct is used to acheive the "Bus" behavior. It is a
//! Multi-producer Multi-consumer channel that is used like a software CAN bus.
//!
//! To process all the messages and data streaming through the bus in an efficient manner, all the structs present 
//! in the GCS-AFV crate use an async-await through architecture [Tokio](https://tokio.rs) to schedule operations as the data is trasmitted and delivered.

/// This module contains the networking struct that enable the bus to operate transparently over different networks
/// such as the Control Stations network and the AFV's local network.
pub mod network;
/// This module contains the structs that initialize and start the Control Stations user interface.
pub mod ui;
/// This module contains the struct that communicate with the AFV's onboard embedded systems such as the 
/// FLIR A50 camera, pan-tilt turrets, pump, lights, and sirens.
pub mod drivers;
/// This module contains the main control systems of the AFV. These communicate directly with the drivers over the bus
/// to enable automatic and manual control of the AFV's systems.
pub mod operators;
/// This module contains the systems that enable the UI on the Control Station to interact with the operators on board the AFV.
pub mod communicators;
