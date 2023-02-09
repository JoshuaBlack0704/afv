use std::sync::Arc;

use gcs_afv::{gui::{TerminalBuilder, GuiArgs}, afv::flir::Flir};


pub struct Args{}
impl GuiArgs for Args{}

fn main(){
    let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not build runtime"));
    let a50 = Flir::actuated_blocking(rt.clone(), None);
    let args = Arc::new(Args{});
    TerminalBuilder::new().add_element(Arc::new(a50)).launch(&args);
}
