use clap::Parser;
use gcs_afv::{network::{ComEngine, AFVPORT}, afv::Afv};
use tokio::time::sleep;


#[derive(Parser, Debug)]
struct Args{
    #[arg(long, default_value_t=AFVPORT)]
    port: u16,
}

#[tokio::main]
async fn main(){
    let args = Args::parse();
    let ip = match local_ip_address::local_ip().expect("Could not get local ip addr") {
        std::net::IpAddr::V4(i) => i,
        std::net::IpAddr::V6(i) => i.to_ipv4_mapped().expect("Could net get ipv4 addr"),
    };
    let com = ComEngine::afv_com_listen(format!("{}:{}", ip, args.port)).await.expect("Could not start com");
    let _afv = Afv::new(com).await;
    loop{
        sleep(tokio::time::Duration::from_secs(10)).await;
    }
}