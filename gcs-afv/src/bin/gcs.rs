use std::sync::Arc;

use gcs_afv::{gui::{GuiArgs, TerminalBuilder}, afvctl::AfvController};

struct Args{}
impl GuiArgs for Args{}

fn main(){
    let args = Arc::new(Args{});
    let ctl = Arc::new(AfvController::new(None));
    TerminalBuilder::new()
    .add_element(ctl)
    .launch(&args);
}
