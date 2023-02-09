use std::net::SocketAddr;
use std::{fmt::Debug, io::Error, mem::size_of, sync::Arc};

use async_trait::async_trait;
use bincode::ErrorKind;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Handle;
use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream, ToSocketAddrs,
    },
    runtime::Runtime,
    sync::{Mutex, RwLock},
    time::{sleep, Duration},
};

use crate::afv::flir::FlirMsg;
use crate::afv::turret::{TurretMsg};

/// The port that gcs's will use to communicate on their network
pub const GCSPORT: u16 = 60000;
/// The port that the ethernet transeivers for the afv's will operate on
pub const AFVPORT: u16 = 4040;
/// How many times an ethernet bus can timeout before it closes
pub const TIMEOUT_BUDGET: u8 = 10;
/// How long till a timeout is issued
pub const TIMEOUT_TIME: Duration = Duration::from_secs(5);

/// The trait that an object must implement should it wish to listen
/// to an ethernet bus
#[async_trait]
pub trait ComEngineService<M>: Send + Sync {
    /// This is the main bus function
    /// Upon receiving a complete msg over the network
    /// an ethernet bus will call this function in a new
    /// async task to let the implementor do what it wants
    async fn notify(self: Arc<Self>, com: Arc<ComEngine<M>>, msg: M);
}

/// The state of a com engine
#[derive(Clone)]
pub enum ComState {
    Active,
    Error(Arc<ComError>),
    Stale,
}

/// The general enum that will be used for communication between the gcs and the afv
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AfvMessage {
    Heart,
    Beat,
    Closed,
    String(String),
    Flir(FlirMsg),
    Turret(TurretMsg),
}

/// What went wrong while creating an ethernet bus
#[derive(Debug)]
pub enum ComError {
    SocketAddr(Error),
    NoAddr,
    CouldNotConnect(Error),
    RecieveError(Error),
    SerializeError(Box<ErrorKind>),
    SendError(Error),
}

/// The ethernet system that will receive and distribute all ethernet communication
pub struct ComEngine<M> {
    state: RwLock<ComState>,
    local_addr: SocketAddr,
    peer_addr: SocketAddr,
    read_socket: Mutex<OwnedReadHalf>,
    write_socket: Mutex<OwnedWriteHalf>,
    listeners: RwLock<Vec<Arc<dyn ComEngineService<M>>>>,
}

impl ComEngine<AfvMessage> {
    /// This will listen for a new connection on addr
    /// when a connection is requested a new ethernet bus will be formed and returned
    pub async fn afv_com_listen(addr: impl ToSocketAddrs) -> Result<Arc<Self>, ComError> {
        let listener = TcpListener::bind(&addr).await.expect("Could not bind addr");
        let (sock, peer_addr) = listener.accept().await.expect("Could not accept socket");
        println!("Accepted socket from {}", peer_addr);
        let (rd, wr) = sock.into_split();
        let local_addr = rd.local_addr().expect("Could not get local addr");
        let com = Arc::new(Self {
            read_socket: Mutex::new(rd),
            write_socket: Mutex::new(wr),
            listeners: RwLock::new(vec![]),
            local_addr,
            peer_addr,
            state: RwLock::new(ComState::Active),
        });

        tokio::spawn(com.clone().listen());

        Ok(com)
    }
    pub fn afv_com_stream(stream: TcpStream) -> Arc<ComEngine<AfvMessage>> {
        let (rd, wr) = stream.into_split();
        let local_addr = rd.local_addr().expect("Could not get local addr");
        let peer_addr = rd.peer_addr().expect("Could not get peer addr");
        let com = Arc::new(Self {
            read_socket: Mutex::new(rd),
            write_socket: Mutex::new(wr),
            listeners: RwLock::new(vec![]),
            local_addr,
            peer_addr,
            state: RwLock::new(ComState::Active),
        });

        tokio::spawn(com.clone().listen());

        com
    }

    pub async fn afv_com(tgt: &impl ToSocketAddrs) -> Result<Arc<Self>, ComError> {
        let sock = match TcpStream::connect(tgt).await {
            Ok(s) => s,
            Err(e) => return Err(ComError::SocketAddr(e)),
        };

        let (rd, wr) = sock.into_split();
        let local_addr = rd.local_addr().expect("Could not get local addr");
        let peer_addr = rd.peer_addr().expect("Could not get peer addr");
        let rd = Mutex::new(rd);
        let wr = Mutex::new(wr);

        let com = Arc::new(Self {
            read_socket: rd,
            write_socket: wr,
            listeners: RwLock::new(vec![]),
            local_addr,
            peer_addr,
            state: RwLock::new(ComState::Active),
        });

        tokio::spawn(com.clone().listen());

        Ok(com)
    }

    pub fn afv_com_blocking(
        rt: &Arc<Runtime>,
        tgt: &impl ToSocketAddrs,
    ) -> Result<Arc<ComEngine<AfvMessage>>, ComError> {
        rt.block_on(Self::afv_com(tgt))
    }

    pub async fn listen(self: Arc<Self>) {
        {
            let reader = self.read_socket.lock().await;
            println!(
                "Tcp streaming on {} to {}",
                reader.local_addr().expect("Could not get local addr"),
                reader.peer_addr().expect("Could not get peer addr")
            );
        }

        let mut data: Vec<u8> = Vec::with_capacity(size_of::<AfvMessage>());
        let mut timeout_budget = TIMEOUT_BUDGET;

        while Arc::strong_count(&self) > 1 && timeout_budget > 0 {
            let preread_length = data.len();
            tokio::select! {
                _ = sleep(TIMEOUT_TIME) => {
                    // println!("Timeout issued");
                    if let Err(e) = self.send(AfvMessage::Heart).await{
                        *self.state.write().await = ComState::Error(Arc::new(e));
                        println!("Send error encounterd");
                        return;
                    }
                    timeout_budget -= 1;
                    continue;
                }
                res = self.process_msg(&mut data) => {
                    if let Err(e) = res{
                        *self.state.write().await = ComState::Error(Arc::new(e));
                        println!("Receive error encounterd");
                        return;
                    }
                }
            }
            
            let postread_length = data.len();
            if preread_length == postread_length {
                println!("EOF recieved");
                break;
            }
            

            let msg: AfvMessage = match bincode::deserialize(&data) {
                Ok(a) => a,
                Err(_) => {
                    continue;
                }
            };
            
            timeout_budget = TIMEOUT_BUDGET;

            if let AfvMessage::Heart = msg {
                if let Err(e) = self.send(AfvMessage::Beat).await {
                    *self.state.write().await = ComState::Error(Arc::new(e));
                    println!("Send error encounterd");
                    return;
                }
                data.clear();
                continue;
            }
            if let AfvMessage::Beat = msg {
                data.clear();
                continue;
            }

            for listener in self.listeners.read().await.iter() {
                tokio::spawn(listener.clone().notify(self.clone(), msg.clone()));
            }

            data.clear();
        }

        *self.state.write().await = ComState::Stale;

        println!(
            "Tcp stream on {} closing",
            self.read_socket
                .lock()
                .await
                .local_addr()
                .expect("No addr for socket")
        );
    }
}

impl<M> ComEngine<M> {
    /// Gets the state of the connection
    pub async fn state(&self) -> ComState {
        self.state.read().await.clone()
    }
    /// Gets the state of the connection
    pub async fn state_blocking(&self) -> ComState {
        self.state.blocking_read().clone()
    }
    /// Adds a new listener to the bus
    pub async fn add_listener(&self, listener: Arc<dyn ComEngineService<M>>) {
        let mut listeners = self.listeners.write().await;
        listeners.push(listener);
    }
    /// Blocks while adding a new listener to the bus
    pub fn add_listener_blocking(&self, listener: Arc<dyn ComEngineService<M>>) {
        let mut listeners = self.listeners.blocking_write();
        listeners.push(listener);
    }
    async fn process_msg(&self, data: &mut Vec<u8>) -> Result<(), ComError> {
        let mut read = self.read_socket.lock().await;
        match read.read_u8().await {
            Ok(byte) => {
                data.push(byte);
                return Ok(());
            }
            Err(e) => {
                return Err(ComError::RecieveError(e));
            }
        }
    }
    async fn send_data(&self, msg: &[u8]) -> Result<(), ComError> {
        let mut write = self.write_socket.lock().await;

        match write.write_all(&msg).await {
            Ok(_) => {
                let _ = write.flush().await;
                return Ok(());
            }
            Err(e) => return Err(ComError::SendError(e)),
        }
    }
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }
}

impl<M: Serialize + Send + Sync + 'static> ComEngine<M> {
    pub async fn send(&self, msg: M) -> Result<(), ComError> {
        let msg = match bincode::serialize(&msg) {
            Ok(d) => d,
            Err(e) => return Err(ComError::SerializeError(e)),
        };
        self.send_data(&msg).await
    }
    pub async fn send_into(self: Arc<Self>, msg: M) -> Result<(), ComError> {
        let msg = match bincode::serialize(&msg) {
            Ok(d) => d,
            Err(e) => return Err(ComError::SerializeError(e)),
        };
        self.send_data(&msg).await
    }
    pub async fn send_parallel(self: Arc<Self>, msg: M) -> Result<(), ComError>{
        let msg = match Handle::current().spawn_blocking(move ||{bincode::serialize(&msg)}).await.expect("Could not join serialize thread"){
            Ok(m) => m,
            Err(e) => return Err(ComError::SerializeError(e)),
        };
        self.send_data(&msg).await
    }
}

/// An empty struct that can be put on an ethernet bus to log traffic
pub struct NetworkLogger {}

#[async_trait]
impl ComEngineService<AfvMessage> for NetworkLogger {
    async fn notify(self: Arc<Self>, _com: Arc<ComEngine<AfvMessage>>, msg: AfvMessage) {
        match msg{
            AfvMessage::Flir(m) => {
                match m{
                    FlirMsg::Nal(_) => {},
                    _ => println!("Network traffic: {:?}", m)
                }
            },
            _ => println!("Network traffic: {:?}", msg),
        }
    }
}

impl NetworkLogger {
    pub async fn afv_com_monitor(bus: &ComEngine<AfvMessage>) {
        let log = Arc::new(Self {});
        bus.add_listener(log).await;
    }
}
