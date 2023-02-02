use gcs_afv::{self, network::{EthernetBus, NetworkLogger, NetworkMessage}};
use tokio::{self, net::{TcpListener, ToSocketAddrs}, io::AsyncWriteExt, time::sleep};

#[tokio::main]
async fn main(){

    let local_addr = "127.0.0.1:4040";
    let join = tokio::spawn(pulse(local_addr.clone()));
    let ethernet = EthernetBus::new(&local_addr).await.expect("Could not connect ethernet bus");
    NetworkLogger::new(&ethernet).await;

    join.await.unwrap();
}

async fn pulse(addr: impl ToSocketAddrs){
    let sleep_time = tokio::time::Duration::from_secs(1);
    for _ in 0..2{
        let listener = TcpListener::bind(&addr).await.expect("Could not create server");
        let (mut sock, _) = listener.accept().await.expect("Could not accept socket");
        let msg = NetworkMessage::Test;
        let msg = bincode::serialize(&msg).unwrap();
        sock.write_all(&msg).await.unwrap();
        sleep(sleep_time).await;
    }
    println!("Done!");
}

