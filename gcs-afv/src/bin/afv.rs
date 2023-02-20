use clap::Parser;
use default_net::get_default_interface;
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
    let ip = get_default_interface().unwrap().ipv4[0].addr;
    let com = ComEngine::afv_com_listen(format!("{}:{}", ip, args.port)).await.expect("Could not start com");
    let _afv = Afv::actuated(com).await;
    loop{
        sleep(tokio::time::Duration::from_secs(10)).await;
    }
}