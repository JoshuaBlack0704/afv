use gcs_afv::network::{NetMessage, afv_bridge::AfvBridge};
use tokio::sync::broadcast;

fn main(){
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not start tokio runtime");
    let (tx, _) = broadcast::channel::<NetMessage>(10000);
    runtime.spawn(AfvBridge::server(tx.clone(), tx.clone(), None));
    let (tx, _) = broadcast::channel::<NetMessage>(10000);
    runtime.block_on(AfvBridge::client(tx.clone(), tx.clone(), Default::default()));
}
