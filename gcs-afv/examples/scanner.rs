use std::sync::Arc;

use gcs_afv::{gui::{TerminalBuilder, GuiArgs}, scanner::Scanner, network::{EthernetBus, NetworkMessage, GCSPORT}};
use tokio::{net::ToSocketAddrs, time::sleep};

pub struct Args{}
impl GuiArgs for Args{}

fn main(){
    let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not build runtime"));
    rt.spawn(pulse());
    let args = Arc::new(Args{});
    let scanner = Arc::new(Scanner::new(Some(rt.clone())));
    TerminalBuilder::new().add_element(scanner).launch(&args);
}

async fn pulse(){
    loop{
        let sleep_time = tokio::time::Duration::from_secs(1);
        let ip = local_ip_address::local_ip().expect("Could not get local ip");
        let ip = match ip{
            std::net::IpAddr::V4(i) => i,
            std::net::IpAddr::V6(i) => i.to_ipv4().unwrap(),
        };
        let ethernet = EthernetBus::server((ip, GCSPORT)).await.expect("Could not run server");
        for i in 0..2{
            let msg = NetworkMessage::String(format!("Hello from server msg {}", i));
            ethernet.send(msg).await;
            sleep(sleep_time).await;
        }
    }
}
