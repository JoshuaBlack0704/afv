use std::sync::Arc;

use default_net::get_default_interface;
use gcs_afv::{gui::{TerminalBuilder, GuiArgs}, scanner::Scanner, network::{ComEngine, AfvMessage, GCSPORT}};
use tokio::time::sleep;

pub struct Args{}
impl GuiArgs for Args{}

fn main(){
    let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not build runtime"));
    rt.spawn(pulse());
    let args = Arc::new(Args{});
    let scanner = Arc::new(Scanner::new_blocking(rt.clone(), None));
    TerminalBuilder::new().add_element(scanner).launch(&args);
}

async fn pulse(){
    loop{
        let sleep_time = tokio::time::Duration::from_secs(1);
        let ip = get_default_interface().unwrap().ipv4[0].addr;
        let ethernet = ComEngine::afv_com_listen((ip, GCSPORT)).await.expect("Could not run server");
        for i in 0..2{
            let msg = AfvMessage::String(format!("Hello from server msg {}", i));
            let _ = ethernet.send(msg).await;
            sleep(sleep_time).await;
        }
    }
}
