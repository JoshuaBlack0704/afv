use std::{collections::HashSet, net::SocketAddr};

use default_net::get_interfaces;
use flume::Sender;
use ipnet::Ipv4Net;
use rand::{thread_rng, Rng};
use tokio::{net::TcpStream, sync::Semaphore};


pub struct ScanBuilder{
    scan_count: ScanCount,
    tgt_ports: Vec<u16>,
    excluded_peers: PeerExclusion,
    parallel_attempts: usize,
}

impl ScanBuilder{
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
        let (tx,rx) = flume::unbounded();
        tokio::spawn(self.scan(tx));
        rx
    }
    async fn scan(self, tx: Sender<TcpStream>){
        // First we pull the interfaces
        let interfaces = get_interfaces();
        // Just used for debugging
        let scan_id = thread_rng().gen::<u16>();
        println!("Scan {} started", scan_id);

        // Now we need to set our scan budget
        let mut scan_budget = match self.scan_count{
            ScanCount::Infinite => 1,
            ScanCount::Limited(l) => l,
        };

        // Prepare our peer exclusion list
        let excluded_peers = match self.excluded_peers{
            PeerExclusion::Never => None,
            PeerExclusion::PreExcluded(p) => {
                let mut set = HashSet::new();
                for p in p{
                    set.insert(p);
                }
                Some(set)
            },
            PeerExclusion::ConnectOnce => Some(HashSet::new()),
        };

        // Now we enter the main while loop
        while scan_budget >= 1{
            // We need to consume a scan
            println!("Scan {}, {} remaining scans", scan_id, scan_budget);
            match self.scan_count{
                ScanCount::Infinite => {},
                ScanCount::Limited(_) => {scan_budget -= 1},
            };

            // We will need the address of each interface to attempt a socket bind
            let mut interface_nets = vec![];
            for interface in interfaces.iter(){
                // We debug its name
                match &interface.friendly_name{
                    Some(n) => println!("Scan {} using interface {}", scan_id, *n),
                    None => println!("Scan {} using interface {}", scan_id, interface.name),
                }
                if let Some(ip) = interface.ipv4.first(){
                    match Ipv4Net::with_netmask(ip.addr, ip.netmask){
                        Ok(net) => {
                            interface_nets.push(net);
                        },
                        Err(_) => {},
                    }
                }
            }

            // We prepare the parallel scan semaphore
            let semaphore = Semaphore::new(self.parallel_attempts);

            // Now we begin the net search
            for net in interface_nets{
                // We can skip loopback since we wont be using it
                if net.addr().is_loopback(){continue;}

                for host in net.hosts(){
                    for &port in self.tgt_ports.iter(){
                        let tgt = SocketAddr::from((host, port));
                        // First we need to see if this ip:port is already excluded
                        match &excluded_peers{
                            Some(p) => {
                                if let Some(_) = p.get(&tgt){continue;}
                            },
                            None => {},
                        }

                        // We will now dispatch a connection task for this target
                    }
                }
            }
        }
        
    }
}

impl Default for ScanBuilder{
    fn default() -> Self {
        Self{
            scan_count: Default::default(),
            excluded_peers: Default::default(),
            tgt_ports: Default::default(),
            parallel_attempts: 500,
        }
    }
}

pub enum ScanCount{
    Infinite,
    Limited(u32),
}
impl Default for ScanCount{
    fn default() -> Self {
        ScanCount::Limited(1)
    }
}

pub enum PeerExclusion{
    Never,
    PreExcluded(Vec<SocketAddr>),
    ConnectOnce,
} 
impl Default for PeerExclusion{
    fn default() -> Self {
        Self::ConnectOnce
    }
}
