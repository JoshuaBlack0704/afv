//! Operators are the main control systems running on the AFV. 
//!
//! Instead of representing the individual components onboard the AFV, the represent the command interfaces
//! that one would use to orchestrate the AFV.

/// This operator is responsible for reading the FLIR A50's IR image stream and performing the
/// fire signature identification algorithm.
///
/// Once the fire analysis data is read it will send the results on the main bus.
///
/// It is enabled only when a stead stream of its auto
/// target messages are appearing on the bus 
///
/// This operator is also responsible for manipulting the FLIR turret when auto targeting is enabled
pub mod flir;

/// This operator is responsible for reading image analysis data, FLIR turret angle information, and Lidar ranging data 
/// to correctly command the nozzle turret to engage the fire.
/// It should also be responsible for turning the pump on when the nozzle turret its in position.
pub mod nozzle;

/// Unused
pub mod pump;

/// Unused
pub mod peripheral;

/// This operator's job is to simple post an AFV's local UUID on the main bus so that any 
/// Control Stations on the bus know of its existence
pub mod naming;

/// This operator is the parent function that initializes the AFV on startup
pub mod afv_launcher;
