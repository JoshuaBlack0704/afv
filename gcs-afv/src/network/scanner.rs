use std::{collections::HashSet, net::SocketAddr, sync::Arc};

use default_net::get_interfaces;
use flume::Sender;
use ipnet::Ipv4Net;
use log::{info, debug};
use rand::{thread_rng, Rng};
use tokio::{
    net::{TcpSocket, TcpStream},
    sync::Semaphore,
    time::{sleep, Duration},
};

pub struct ScanBuilder {
    scan_count: ScanCount,
    tgt_ports: Vec<u16>,
    excluded_peers: PeerExclusion,
    parallel_attempts: usize,
    wait_time: Duration,
}

impl ScanBuilder {
    pub fn scan_count(mut self, scan_count: ScanCount) -> ScanBuilder {
        self.scan_count = scan_count;
        self
    }
    pub fn excluded_peers(mut self, peers: PeerExclusion) -> ScanBuilder {
        self.excluded_peers = peers;
        self
    }
    pub fn parallel_attempts(mut self, parallel_attempts: usize) -> ScanBuilder {
        self.parallel_attempts = parallel_attempts;
        self
    }
    pub fn add_port(mut self, port: u16) -> ScanBuilder {
        self.tgt_ports.push(port);
        self
    }
    /// Must be called within an async runtime
    pub fn dispatch(self) -> flume::Receiver<TcpStream> {
        let (tx, rx) = flume::unbounded();
        tokio::spawn(self.scan(tx));
        rx
    }
    async fn scan(self, tx: Sender<TcpStream>) {
        // First we pull the interfaces
        let mut interfaces = match default_net::get_default_interface(){
            Ok(i) => vec![i],
            Err(_) => vec![],
        };
        'outer: for new_i in get_interfaces(){
            for old_i in interfaces.iter(){
                if new_i.name == old_i.name{
                    continue 'outer;
                }
            }
            interfaces.push(new_i);
        }

        // Just used for debugging
        let scan_id = thread_rng().gen::<u16>();

        // Now we need to set our scan budget
        let mut scan_budget = match self.scan_count {
            ScanCount::Infinite => 1,
            ScanCount::Limited(l) => l,
        };

        // Prepare our peer exclusion list
        let excluded_peers = match self.excluded_peers {
            PeerExclusion::Never => None,
            PeerExclusion::PreExcluded(p) => {
                let mut set = HashSet::new();
                for p in p {
                    set.insert(p);
                }
                Some(set)
            }
            PeerExclusion::ConnectOnce => Some(HashSet::new()),
        };

        // Now we enter the main while loop
        while scan_budget >= 1 {
            // We need to consume a scan
            match self.scan_count {
                ScanCount::Infinite => {
                    info!("Scan {} running", scan_id);
                }
                ScanCount::Limited(_) => {
                    info!("Scan {}, {} remaining scans", scan_id, scan_budget);
                    scan_budget -= 1;
                }
            };

            // We will need the address of each interface to attempt a socket bind
            let mut interface_nets = vec![];
            for interface in interfaces.iter() {
                if let Some(ip) = interface.ipv4.first() {
                    if let Ok(net) = Ipv4Net::with_netmask(ip.addr, ip.netmask) {
                        // We can skip loopback since we wont be using it
                        if net.addr().is_loopback() {
                            continue;
                        }
                        // We debug its name
                        match &interface.friendly_name {
                            Some(n) => debug!(
                                "Scan {} using interface {} with address {}",
                                scan_id,
                                *n,
                                net.addr()
                            ),
                            None => debug!(
                                "Scan {} using interface {} with address {}",
                                scan_id,
                                interface.name,
                                net.addr()
                            ),
                        }
                        interface_nets.push(net);
                    }
                }
            }

            // We prepare the parallel scan semaphore
            let semaphore = Arc::new(Semaphore::new(self.parallel_attempts));
            // The success channel
            let (socket_tx, socket_rx) = flume::unbounded();
            // For debugging
            let mut task_count = 0;

            // Now we begin the net search
            for net in interface_nets {
                for host in net.hosts() {
                    for &port in self.tgt_ports.iter() {
                        let tgt = SocketAddr::from((host, port));
                        // First we need to see if this ip:port is already excluded
                        match &excluded_peers {
                            Some(p) => {
                                if let Some(_) = p.get(&tgt) {
                                    continue;
                                }
                            }
                            None => {}
                        }

                        // We will now dispatch a connection task for this target
                        let tx = tx.clone();
                        let socket_tx = socket_tx.clone();
                        let semaphore = semaphore.clone();
                        task_count += 1;

                        tokio::spawn(async move {
                            let _signal = socket_tx;
                            let _permit = match semaphore.acquire().await {
                                Ok(p) => p,
                                Err(_) => return,
                            };
                            if tx.is_disconnected() {
                                return;
                            }
                            let socket = match TcpSocket::new_v4() {
                                Ok(s) => s,
                                Err(_) => return,
                            };
                            if let Err(_) = socket.bind((net.addr(), 0u16).into()) {
                                return;
                            }
                            if let Ok(s) = socket.connect(tgt).await {
                                if let Ok(addr) = s.peer_addr() {
                                    info!("Scanner found peer at {}", addr)
                                }
                                let _ = tx.send_async(s).await;
                            }
                        });
                    }
                }
            }

            debug!("Scan {} started {} connect tasks", scan_id, task_count);

            drop(socket_tx);
            // This will complete with an error when the last _signal is dropped in
            // the socket connect tasks
            let _: Result<bool, flume::RecvError> = socket_rx.recv_async().await;

            // We are now finished with a scan round
            // we will wait for the specified wait time
            info!("Scan {} round completed", scan_id);
            if tx.is_disconnected() {
                return;
            }
            sleep(self.wait_time).await;
        }
    }
}

impl Default for ScanBuilder {
    fn default() -> Self {
        Self {
            scan_count: Default::default(),
            excluded_peers: Default::default(),
            tgt_ports: Default::default(),
            parallel_attempts: 1000,
            wait_time: Duration::from_secs(5),
        }
    }
}

pub enum ScanCount {
    Infinite,
    Limited(u32),
}
impl Default for ScanCount {
    fn default() -> Self {
        ScanCount::Limited(1)
    }
}

pub enum PeerExclusion {
    Never,
    PreExcluded(Vec<SocketAddr>),
    ConnectOnce,
}
impl Default for PeerExclusion {
    fn default() -> Self {
        Self::ConnectOnce
    }
}
