use std::{sync::Arc, net::SocketAddr, marker::PhantomData};

use async_trait::async_trait;
use clap::Parser;
use futures::StreamExt;
use rand::{thread_rng, Rng};
use retina::{client::{PlayOptions, SetupOptions, SessionOptions, self}, codec::CodecItem::VideoFrame};
use tokio::{time::{sleep, Instant, Duration}, runtime::Handle, sync::RwLock};
use url::Url;

use crate::{bus::{Bus, BusUuid, BusElement}, networkbus::{networkbridge::NetworkBridge, scanner::ScanBuilder}, GCSBRIDGEPORT, messages::{AfvCtlMessage, NetworkMessages, LocalMessages}};

#[derive(Parser, Debug)]
pub struct AfvArgs{
    connect: Option<SocketAddr>,
}

pub type AfvUuid = u16;
pub const FLIR_TIME: Duration = Duration::from_secs(3);
pub struct Simulated;
pub struct Actuated;
// pub const MINSTREAMINSTANTDIFF = 
pub struct Afv<SimType>{
    // General fields
    bus_uuid: BusUuid,
    afv_uuid: AfvUuid,
    bus: Bus<AfvCtlMessage>,
    handle: Handle,
    _sim: PhantomData<SimType>,

    // Flir fields
    flir_net_request: RwLock<Instant>,
    flir_local_request: RwLock<Instant>,
}

impl Afv<Simulated>{
    pub async fn simulate(){
        let bus = Bus::<AfvCtlMessage>::new().await;
        println!("Afv listening on port {}", GCSBRIDGEPORT);
        NetworkBridge::server(&bus, GCSBRIDGEPORT).await;
        println!("Afv connected on port {}", GCSBRIDGEPORT);
        let afv:Arc<Afv<Simulated>> = Arc::new(Afv{
            bus_uuid: thread_rng().gen(),
            afv_uuid: thread_rng().gen(),
            bus: bus.clone(),
            handle: Handle::current(),
            _sim: PhantomData,
            flir_net_request: RwLock::new(Instant::now()),
            flir_local_request: RwLock::new(Instant::now()),
        });
        
        tokio::spawn(afv.clone().stream_flir());

        bus.add_element(afv.clone()).await;
        
        loop{
            sleep(Duration::from_secs(10)).await;
        }
        
    }
}

impl Afv<Actuated>{
    pub fn actuate(){
        let args = AfvArgs::parse();
        let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not build runtime"));
        rt.block_on(async move {
            let bus = Bus::<AfvCtlMessage>::new().await;
            match args.connect{
                Some(addr) => {
                    println!("Afv connecting");
                    NetworkBridge::client(&bus, &addr).await;
                    println!("Afv connected");
                },
                None => {
                    println!("Afv listening on port {}", GCSBRIDGEPORT);
                    NetworkBridge::server(&bus, GCSBRIDGEPORT).await;
                    println!("Afv connected on port {}", GCSBRIDGEPORT);
                },
            }
            let afv:Arc<Afv<Simulated>> = Arc::new(Afv{
                bus_uuid: thread_rng().gen(),
                afv_uuid: thread_rng().gen(),
                bus: bus.clone(),
                handle: Handle::current(),
                _sim: PhantomData,
                flir_net_request: RwLock::new(Instant::now()),
                flir_local_request: RwLock::new(Instant::now()),
            });

            tokio::spawn(afv.clone().stream_flir());

            bus.add_element(afv.clone()).await;
            
            loop{
                sleep(Duration::from_secs(10)).await;
            }
        });
    }
}

impl<T> Afv<T>{
    async fn stream_flir(self: Arc<Self>){
        // The first step is to attempt a connection to the flir
        let scan = ScanBuilder::default()
        .scan_count(crate::networkbus::scanner::ScanCount::Infinite)
        .add_port(554)
        .dispatch();
        println!("Started flir scan");

        // We will not go further until we have found a flir
        let flir_ip = match scan.recv_async().await{
            Ok(flir) => {
                match flir.peer_addr(){
                    Ok(ip) => ip,
                    Err(_) => return,
                }
            },
            Err(_) => return,
        };

        // Stop the scan
        drop(scan);

        // Now that we have found a flir we can start the stream
        let url = match Url::parse(&format!("rtsp://:@{}:554/avc", flir_ip.ip())){
            Ok(u) => {
               u 
            },
            Err(_) => return,
        };

        
        let mut options = SessionOptions::default();
        options = options.user_agent(String::from("demo"));

        let mut session = match client::Session::describe(url, options).await {
            Ok(s) => s,
            Err(_) => return,
        };

        let options = SetupOptions::default();
        if let Err(_) = session.setup(0, options).await {
            return;
        }

        let options = PlayOptions::default();
        let play = match session.play(options).await {
            Ok(p) => p,
            Err(_) => return,
        };

        let demux = match play.demuxed() {
            Ok(d) => d,
            Err(_) => return,
        };
        
        println!("FLIR ACTUATOR: Rtsp stream opened on {}", flir_ip);
        
        tokio::pin!(demux);

        // Now that we have a stream we can begin to pull NAL packets out

        loop{
            let frame;
            match demux.next().await{
                Some(f) => {
                    // We only care about video frames
                    if let Ok(VideoFrame(v)) = f{
                        frame = v.into_data();
                    }
                    else{
                        continue;
                    }
                },
                None => {continue;},
            }

            if Instant::now().duration_since(*self.flir_net_request.read().await).is_zero(){
                // The command instant has not passed, meaning we have charge to send a nal packet
                
                self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Network(NetworkMessages::NalPacket(self.afv_uuid, frame.clone()))).await;
            }
            if Instant::now().duration_since(*self.flir_local_request.read().await).is_zero(){
                // The command instant has not passed, meaning we have charge to send a nal packet
                self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Local(LocalMessages::NalPacket(self.afv_uuid, frame))).await;
            }
        }
    }
    async fn flir_net_request(self: Arc<Self>){
        if let Some(i) = Instant::now().checked_add(FLIR_TIME){
            *self.flir_net_request.write().await = i;
        }
    }
    async fn flir_local_request(self: Arc<Self>){
        if let Some(i) = Instant::now().checked_add(FLIR_TIME){
            *self.flir_local_request.write().await = i;
        }
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> BusElement<AfvCtlMessage> for Afv<T>{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){
        if let AfvCtlMessage::Network(msg) = msg{
            match msg{
                NetworkMessages::PollAfvUuid => {
                    // This message type is handled the same
                    tokio::spawn(
                        self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Network(NetworkMessages::AfvUuid(self.afv_uuid)))
                    );
                },
                NetworkMessages::FlirStream(afv_uuid) => {
                    if self.afv_uuid != afv_uuid{return;}
                    tokio::spawn(self.clone().flir_net_request());
                },
                _ => {}
            }
            return
        }

        if let AfvCtlMessage::Local(msg) = msg{
            match msg{
                LocalMessages::FlirStream(afv_uuid) => {
                    if self.afv_uuid != afv_uuid{return;}
                    tokio::spawn(self.clone().flir_local_request());
                }
                _ => {}
            }
            return;
        }
        
    }
    fn uuid(&self) -> BusUuid{
        self.bus_uuid
    }
}