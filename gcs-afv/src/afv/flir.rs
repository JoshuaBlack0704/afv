use async_trait::async_trait;
use image::DynamicImage;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FlirMsg{
    
}

#[async_trait]
pub trait IrSource{
    /// Will return a complete nal frame using the retina avc demuxer
    async fn next(&self) -> Vec<u8>;
    /// Will return a complete rgb image by polling the ir cam via
    /// next until a successful decode is achieved
    async fn image(&self) -> DynamicImage;
}

/// The driver for the Flir A50
pub struct A50<S:IrSource>{
    source: S,
}

/// Will attempt to establish a RTSP session with a flir camera
pub struct RtspSession{
    
}

/// Will conduct communication over the network to gather data needed for 
/// ir image reconstruction
pub struct A50Link{
    
}

