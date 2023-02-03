use std::{fmt::Debug, io::Error, mem::size_of, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpSocket;
use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream, ToSocketAddrs,
    },
    runtime::Runtime,
    sync::{Mutex, RwLock},
    time::{sleep, Duration},
};

/// The port that gcs's will use to communicate on their network
pub const GCSPORT: u16 = 60000;
/// The port that the ethernet transeivers for the afv's will operate on
pub const AFVPORT: u16 = 4040;
/// How many times an ethernet bus can timeout before it closes
pub const TIMEOUT_BUDGET: u8 = 10;

/// The trait that an object must implement should it wish to listen
/// to an ethernet bus
#[async_trait]
pub trait EthernetListener<M>: Send + Sync {
    /// This is the main bus function
    /// Upon receiving a complete msg over the network
    /// an ethernet bus will call this function in a new
    /// async task to let the implementor do what it wants
    async fn notify(self: Arc<Self>, msg: M);
}

/// The general enum that will be used for communication between the gcs and the afv
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    Heart,
    Beat,
    Empty,
    String(String),
}

/// What went wrong while creating an ethernet bus
#[derive(Debug)]
pub enum EthernetBusError {
    SocketAddr(Error),
    NoAddr,
    CouldNotConnect(Error),
}

/// The ethernet system that will receive and distribute all ethernet communication
pub struct EthernetBus<M> {
    read_socket: Mutex<OwnedReadHalf>,
    write_socket: Mutex<OwnedWriteHalf>,
    listeners: RwLock<Vec<Arc<dyn EthernetListener<M>>>>,
}

/// An empty struct that can be put on an ethernet bus to log traffic
pub struct NetworkLogger {}

#[async_trait]
impl EthernetListener<NetworkMessage> for NetworkLogger {
    async fn notify(self: Arc<Self>, msg: NetworkMessage) {
        println!("Network traffic: {:?}", msg);
    }
}

impl NetworkLogger {
    pub async fn new(bus: &EthernetBus<NetworkMessage>) {
        let log = Arc::new(Self {});
        bus.add_listener(log).await;
    }
}

impl<M> EthernetBus<M> {
    /// Adds a new listener to the bus
    pub async fn add_listener(&self, listener: Arc<dyn EthernetListener<M>>) {
        let mut listeners = self.listeners.write().await;
        listeners.push(listener);
    }
    /// Blocks while adding a new listener to the bus
    pub fn add_listener_blocking(&self, listener: Arc<dyn EthernetListener<M>>) {
        let mut listeners = self.listeners.blocking_write();
        listeners.push(listener);
    }
}

impl EthernetBus<NetworkMessage> {
    /// This will listen for a new connection on addr
    /// when a connection is requested a new ethernet bus will be formed and returned
    pub async fn server(
        addr: impl ToSocketAddrs,
    ) -> Result<Arc<EthernetBus<NetworkMessage>>, EthernetBusError> {
        let listener = TcpListener::bind(&addr).await.expect("Could not bind addr");
        let (sock, peer_addr) = listener.accept().await.expect("Could not accept socket");
        println!("Accepted socket from {}", peer_addr);
        let (rd, wr) = sock.into_split();
        let ethernet = Arc::new(Self {
            read_socket: Mutex::new(rd),
            write_socket: Mutex::new(wr),
            listeners: RwLock::new(vec![]),
        });

        tokio::spawn(ethernet.clone().listen());

        Ok(ethernet)
    }

    /// Attempts to connect to tgt, returning an ethernet bus when successful
    pub async fn new(
        tgt: &impl ToSocketAddrs,
    ) -> Result<Arc<EthernetBus<NetworkMessage>>, EthernetBusError> {
        let sock = match TcpStream::connect(tgt).await {
            Ok(s) => s,
            Err(e) => return Err(EthernetBusError::SocketAddr(e)),
        };

        let (rd, wr) = sock.into_split();
        let rd = Mutex::new(rd);
        let wr = Mutex::new(wr);

        let ethernet = Arc::new(Self {
            read_socket: rd,
            write_socket: wr,
            listeners: RwLock::new(vec![]),
        });

        tokio::spawn(ethernet.clone().listen());

        Ok(ethernet)
    }
    pub fn new_blocking(
        runtime: Arc<Runtime>,
        tgt: &impl ToSocketAddrs,
    ) -> Result<Arc<EthernetBus<NetworkMessage>>, EthernetBusError> {
        runtime.block_on(Self::new(tgt))
    }

    /// The main network task of the ethernet bus
    /// With receive bytes and decode them into NetworkMessages
    /// Then will notify all bus participants
    async fn listen(self: Arc<Self>) {
        {
            let reader = self.read_socket.lock().await;
            println!(
                "Tcp streaming on {} to {}",
                reader.local_addr().expect("Could not get local addr"),
                reader.peer_addr().expect("Could not get peer addr")
            );
        }
        let sleep_time = Duration::from_secs(5);
        let mut data = Vec::with_capacity(size_of::<NetworkMessage>());
        let mut timeout_budget = TIMEOUT_BUDGET;

        while Arc::strong_count(&self) > 1 && timeout_budget > 0 {
            let preread_length = data.len();
            tokio::select! {
                _ = sleep(sleep_time) => {
                    println!("Timeout");
                    self.send(NetworkMessage::Heart).await;
                    timeout_budget -= 1;
                    continue;
                }
                _ = self.process_msg(&mut data) => {}
            }

            timeout_budget = TIMEOUT_BUDGET;
            let postread_length = data.len();
            if preread_length == postread_length {
                break;
            }

            let msg: NetworkMessage = match bincode::deserialize::<NetworkMessage>(&data) {
                Ok(a) => a,
                Err(_) => {
                    continue;
                }
            };

            if let NetworkMessage::Heart = msg {
                println!("Heart recived, beating");
                self.send(NetworkMessage::Beat).await;
            }
            if let NetworkMessage::Beat = msg {
                continue;
            }

            for listener in self.listeners.read().await.iter() {
                tokio::spawn(listener.clone().notify(msg.clone()));
            }

            data.clear();
        }

        println!(
            "Tcp stream on {} closing",
            self.read_socket
                .lock()
                .await
                .local_addr()
                .expect("No addr for socket")
        );
    }

    /// Use the ethernet bus's internal writer to send a msg
    pub async fn send(&self, msg: NetworkMessage) {
        let msg = bincode::serialize(&msg).expect("Could not serialize msg");
        let mut write = self.write_socket.lock().await;
        if let Ok(_) = write.write_all(&msg).await {
            let _ = write.flush().await;
        }
    }

    /// The task that handles receiving a byte
    async fn process_msg(&self, data: &mut Vec<u8>) {
        let mut read = self.read_socket.lock().await;
        if let Ok(byte) = read.read_u8().await {
            data.push(byte);
        }
    }
}
