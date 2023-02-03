use std::sync::Arc;

use gcs_afv::{gui::{TerminalBuilder, GuiArgs}, scanner::Scanner};

pub struct Args{}
impl GuiArgs for Args{}

fn main(){
    let args = Arc::new(Args{});
    let scanner = Arc::new(Scanner::new(None));
    TerminalBuilder::new().add_element(scanner).launch(&args);
}
