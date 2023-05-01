use std::{net::SocketAddr, sync::Arc};

use log::info;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    sync::{Mutex, MutexGuard},
};

/// Will handle maintaing a tcp socket as well as provide procisions
/// for sending and receiving data
#[derive(Clone)]
pub struct Socket {
    server: bool,
    rd: Arc<Mutex<OwnedReadHalf>>,
    wr: Arc<Mutex<OwnedWriteHalf>>,
    peer: SocketAddr,
    local: SocketAddr,
}

impl Socket {
    pub fn new(stream: TcpStream, server: bool) -> Socket {
        let peer = stream.peer_addr().unwrap();
        let ip = stream.local_addr().unwrap();
        let (rd, wr) = stream.into_split();

        Self {
            rd: Arc::new(Mutex::new(rd)),
            wr: Arc::new(Mutex::new(wr)),
            peer,
            server,
            local: ip,
        }
    }

    pub async fn get_reader(&self) -> MutexGuard<OwnedReadHalf> {
        self.rd.lock().await
    }
    pub async fn get_writer(&self) -> MutexGuard<OwnedWriteHalf> {
        self.wr.lock().await
    }
    pub async fn read_byte(&self) -> u8 {
        loop {
            let mut rd = self.get_reader().await;

            match rd.read_u8().await {
                Ok(byte) => return byte,
                Err(_) => {
                    drop(rd);
                    self.clone().reconnect().await;
                }
            }
        }
    }
    pub async fn write_data(&self, data: &[u8]) {
        let _ = self.get_writer().await.write(data).await;
    }
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer
    }
    pub fn local_addr(&self) -> SocketAddr {
        self.local
    }
    async fn reconnect(&self) {
        info!("Socket has disconnected from {}", self.peer);
        let mut rd = self.get_reader().await;
        let mut wr = self.get_writer().await;

        loop {
            if self.server {
                if let Ok(l) = TcpListener::bind((self.local.ip(), self.local.port())).await {
                    if let Ok((s, _)) = l.accept().await {
                        info!("Socket server has reconnected to {}", self.peer);
                        (*rd, *wr) = s.into_split();
                        return;
                    }
                }
                continue;
            }
            match TcpStream::connect(self.peer).await {
                Ok(s) => {
                    info!("Socket client has reconnected to {}", self.peer);
                    (*rd, *wr) = s.into_split();
                    return;
                }
                Err(_) => {}
            }
        }
    }
}
