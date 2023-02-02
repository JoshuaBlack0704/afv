use gcs_afv::{self, network::{EthernetBus, NetworkLogger, NetworkMessage, GCSPORT}};
use tokio::{self, net::{TcpListener, ToSocketAddrs}, io::AsyncWriteExt, time::sleep};

#[tokio::main]
async fn main(){

    let local_addr = format!("127.0.0.1:{}", GCSPORT);
    let join = tokio::spawn(pulse(local_addr.clone()));
    let ethernet = EthernetBus::new(&local_addr).await.expect("Could not connect ethernet bus");
    NetworkLogger::new(&ethernet).await;

    join.await.unwrap();
}

async fn pulse(addr: impl ToSocketAddrs){
    let sleep_time = tokio::time::Duration::from_secs(1);
    let ethernet = EthernetBus::server(addr).await.expect("Could not run server");
    for i in 0..2{
        let msg = NetworkMessage::String(format!("Hello from server msg {}", i));
        ethernet.send(msg).await;
        sleep(sleep_time).await;
    }
    println!("Done!");
}

