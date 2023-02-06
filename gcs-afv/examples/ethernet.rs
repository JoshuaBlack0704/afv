use gcs_afv::{self, network::{ComEngine, NetworkLogger, GCSPORT, AfvMessage}};
use tokio::{self, net::ToSocketAddrs, time::sleep};

#[tokio::main]
async fn main(){

    let local_addr = format!("127.0.0.1:{}", GCSPORT);
    let join = tokio::spawn(pulse(local_addr.clone()));
    let ethernet = ComEngine::afv_com(&local_addr).await.expect("Could not connect ethernet bus");
    NetworkLogger::afv_com_monitor(&ethernet).await;

    join.await.unwrap();
}

async fn pulse(addr: impl ToSocketAddrs){
    let sleep_time = tokio::time::Duration::from_secs(1);
    let ethernet = ComEngine::afv_com_listen(addr).await.expect("Could not run server");
    for i in 0..2{
        let msg = AfvMessage::String(format!("Hello from server msg {}", i));
        let _ = ethernet.send(msg).await;
        sleep(sleep_time).await;
    }
    println!("Done!");
}

