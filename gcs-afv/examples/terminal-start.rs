use std::sync::Arc;

use gcs_afv::gui::{TerminalBuilder, Tutorial, GuiArgs};

pub struct Args{}
impl GuiArgs for Args{}

fn main(){
    let args = Arc::new(Args{});
    let tutorial = Tutorial::new();
    TerminalBuilder::new()
    .add_element(tutorial)
    .launch(&args);
}
