use std::sync::Arc;

use gcs_afv::{afvctl::AfvController, gui::{TerminalBuilder, GuiArgs}};

struct Args{}
impl GuiArgs for Args{}

fn main(){
    let args = Arc::new(Args{});
    let ctl = Arc::new(AfvController::new(None));
    ctl.spawn_dummy();
    TerminalBuilder::new()
    .add_element(ctl)
    .launch(&args);
}
