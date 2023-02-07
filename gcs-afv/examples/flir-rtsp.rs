use std::sync::Arc;

use gcs_afv::{gui::{TerminalBuilder, GuiArgs}, afv::flir::{RtspSession, A50}};
use tokio::time::Duration;

pub struct Args{}
impl GuiArgs for Args{}

fn main(){
    let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not build runtime"));
    let rtsp = RtspSession::new_blocking(rt.clone());
    let a50 = A50::new(Some(rt), Arc::new(rtsp.clone()));
    a50.clone().refresh_interval(Duration::from_millis(100));
    let args = Arc::new(Args{});
    TerminalBuilder::new().add_element(a50).launch(&args);
}
