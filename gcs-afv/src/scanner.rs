use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use async_trait::async_trait;
use eframe::egui::{self, Ui};
use tokio::{
    net::TcpStream,
    runtime::Runtime,
    sync::{Mutex, RwLock, Semaphore},
};

use crate::{gui::GuiElement, network::AFVPORT};

#[derive(Clone, Copy)]
pub enum ScannerState {
    Available,
    Dispatched,
    Complete,
}

#[async_trait]
pub trait ScannerAddrHandler: Send + Sync {
    async fn handle(&self, stream: TcpStream);
}

pub struct Scanner {
    rt: Arc<Runtime>,
    gateway: RwLock<Ipv4Addr>,
    subnet: RwLock<Ipv4Addr>,
    port_range: RwLock<(u16, u16)>,
    state: RwLock<ScannerState>,
    open: RwLock<bool>,
    target_count: RwLock<usize>,
    connects: RwLock<Vec<SocketAddr>>,
    parallel_attempts: RwLock<u32>,
    semaphore: Mutex<Option<Arc<Semaphore>>>,
    handler: RwLock<Option<Arc<dyn ScannerAddrHandler>>>,
}

impl Scanner {
    pub fn new(rt: Option<Arc<Runtime>>) -> Arc<Self> {
        let rt = match rt {
            Some(rt) => rt,
            None => Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Could not construct runtime for scanenr"),
            ),
        };

        let ip = match local_ip_address::local_ip() {
            Ok(ip) => {
                let mut res = Ipv4Addr::new(192, 168, 1, 1);
                if let IpAddr::V4(i) = ip {
                    res = i
                }
                res
            }
            Err(_) => Ipv4Addr::new(192, 168, 1, 1),
        };

        Arc::new(Self {
            gateway: RwLock::new(ip),
            subnet: RwLock::new(Ipv4Addr::new(255, 255, 255, 0)),
            port_range: RwLock::new((AFVPORT, AFVPORT)),
            state: RwLock::new(ScannerState::Available),
            rt,
            open: RwLock::new(false),
            target_count: RwLock::new(0),
            parallel_attempts: RwLock::new(100),
            semaphore: Mutex::new(None),
            handler: RwLock::new(None),
            connects: RwLock::new(vec![]),
        })
    }
    pub async fn set_handler(&self, handler: Arc<dyn ScannerAddrHandler>) {
        *self.handler.write().await = Some(handler);
    }
    pub fn set_handler_blocking(&self, handler: Arc<dyn ScannerAddrHandler>) {
        self.rt.block_on(self.set_handler(handler));
    }

    pub async fn cancel_scan(&self) {
        let mut semaphore = self.semaphore.blocking_lock();
        let mut closed = false;
        if let Some(s) = &mut (*semaphore) {
            s.close();
            closed = true;
        }
        if closed {
            *semaphore = None;
        }
    }
    pub fn cancel_scan_blocking(&self) {
        self.rt.block_on(self.cancel_scan());
    }
    pub fn new_with_config<F: Fn(SocketAddr) + Send + Sync + 'static>(
        rt: Option<Arc<Runtime>>,
        gateway: Ipv4Addr,
        subnet: Ipv4Addr,
        port_range: (u16, u16),
        parallel_attempts: u32,
    ) -> Arc<Self> {
        let scanner = Self::new(rt);
        *scanner.gateway.blocking_write() = gateway;
        *scanner.subnet.blocking_write() = subnet;
        *scanner.port_range.blocking_write() = port_range;
        *scanner.parallel_attempts.blocking_write() = parallel_attempts;
        scanner
    }
    pub fn ui(self: &Arc<Self>, ui: &mut Ui) {
        let state = *self.state.blocking_read();

        match state {
            ScannerState::Available => {
                self.available_ui(ui);
            }
            ScannerState::Dispatched => {
                self.dispatched_ui(ui);
            }
            ScannerState::Complete => {
                self.completed_ui(ui);
            }
        }
    }
    fn available_ui(self: &Arc<Self>, ui: &mut Ui) {
        let mut gateway = self.gateway.blocking_write();
        let mut subnet = self.subnet.blocking_write();
        let mut port_range = self.port_range.blocking_write();
        let mut attempts = self.parallel_attempts.blocking_write();

        let mut g_octets = gateway.octets();
        let mut s_octets = subnet.octets();
        ui.vertical_centered(|ui| {
            egui::Grid::new("Scanner options")
                .num_columns(5)
                .spacing([5.0, 5.0])
                .striped(true)
                .show(ui, |ui| {
                    // Ip addr
                    ui.label("Gateway Ip: ");
                    let drag_val = egui::DragValue::new(&mut g_octets[0]).clamp_range(0..=255);
                    ui.add(drag_val);
                    let drag_val = egui::DragValue::new(&mut g_octets[1]).clamp_range(0..=255);
                    ui.add(drag_val);
                    let drag_val = egui::DragValue::new(&mut g_octets[2]).clamp_range(0..=255);
                    ui.add(drag_val);
                    let drag_val = egui::DragValue::new(&mut g_octets[3]).clamp_range(0..=255);
                    ui.add(drag_val);
                    *gateway = Ipv4Addr::from(g_octets);
                    ui.end_row();
                    // subnet
                    ui.label("Subnet mask: ");
                    let drag_val = egui::DragValue::new(&mut s_octets[0]).clamp_range(0..=255);
                    ui.add(drag_val);
                    let drag_val = egui::DragValue::new(&mut s_octets[1]).clamp_range(0..=255);
                    ui.add(drag_val);
                    let drag_val = egui::DragValue::new(&mut s_octets[2]).clamp_range(0..=255);
                    ui.add(drag_val);
                    let drag_val = egui::DragValue::new(&mut s_octets[3]).clamp_range(0..=255);
                    ui.add(drag_val);
                    *subnet = Ipv4Addr::from(s_octets);
                    ui.end_row();
                    // ports
                    ui.label("Port Range: ");
                    let drag_val =
                        egui::DragValue::new(&mut port_range.0).clamp_range(0..=u16::MAX);
                    ui.add(drag_val);
                    let drag_val =
                        egui::DragValue::new(&mut port_range.1).clamp_range(0..=u16::MAX);
                    ui.add(drag_val);
                    ui.end_row();

                    // Concurrency
                    ui.label("Concurrent attemps");
                    let drag_val = egui::DragValue::new(&mut (*attempts)).clamp_range(1..=u32::MAX);
                    ui.add(drag_val)
                });

            let port_count: u64 = (port_range.0..=port_range.1).count() as u64;
            let mut subnet_number = u32::from_be_bytes(s_octets);
            let mut subnet_bits: u32 = 0;

            for _ in 0..u32::BITS {
                if subnet_number & 1 == 0 {
                    subnet_bits += 1;
                }
                subnet_number >>= 1;
            }

            let count = port_count * 2u64.pow(subnet_bits);
            let count = format!("Total targets: {}", count);

            ui.label(count);
            if ui.button("Loopback").clicked() {
                *gateway = Ipv4Addr::new(127, 0, 0, 1);
                *subnet = Ipv4Addr::new(255, 255, 255, 255);
            }
            if ui.button("Start").clicked() {
                self.rt.spawn(self.clone().dispatch());
            };
        });
    }
    fn dispatched_ui(self: &Arc<Self>, ui: &mut Ui) {
        let gateway = self.gateway.blocking_read().octets();
        let subnet = self.subnet.blocking_read().octets();
        let port_range = *self.port_range.blocking_read();

        let gateway = format!(
            "Gatway: {}.{}.{}.{}",
            gateway[0], gateway[1], gateway[2], gateway[3]
        );
        let subnet = format!(
            "Subnet: {}.{}.{}.{}",
            subnet[0], subnet[1], subnet[2], subnet[3]
        );
        let port_range = format!("Port Range: {}-{}", port_range.0, port_range.1);
        let targets_remaining =
            format!("Remaining targets: {}", *self.target_count.blocking_read());
        let successful_connects = format!(
            "Successful connections: {}",
            self.connects.blocking_read().len()
        );

        ui.vertical_centered(|ui| {
            ui.label(gateway);
            ui.label(subnet);
            ui.label(port_range);
            ui.label(targets_remaining);
            ui.label(successful_connects);
            let mut semaphore = self.semaphore.blocking_lock();
            let mut closed = false;
            if let Some(s) = &mut (*semaphore) {
                if ui.button("Cancel scan").clicked() {
                    s.close();
                    closed = true;
                }
            } else {
                ui.label("Canceling scan...");
            }
            if closed {
                *semaphore = None;
            }
        });
    }
    fn completed_ui(self: &Arc<Self>, ui: &mut Ui) {
        let targets_remaining = format!("targets scanned: {}", *self.target_count.blocking_read());
        let successful_connects = format!(
            "Successful connections: {}",
            self.connects.blocking_read().len()
        );
        ui.vertical_centered(|ui| {
            ui.label(targets_remaining);
            ui.label(successful_connects);
            if ui.button("Restart").clicked() {
                *self.state.blocking_write() = ScannerState::Available;
                *self.target_count.blocking_write() = 0;
                self.connects.blocking_write().clear();
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                for con in self.connects.blocking_read().iter() {
                    ui.label(format!("{}", con));
                }
            });
        });
    }
    async fn dispatch(self: Arc<Self>) {
        *self.state.write().await = ScannerState::Dispatched;
        let gateway = self.gateway.read().await.octets();
        let subnet = self.subnet.read().await.octets();
        let port_range = *self.port_range.read().await;
        let port_count = (port_range.0..=port_range.1).count();
        let concurrent_attempts = *self.parallel_attempts.read().await;

        let mut octet_matches = [vec![], vec![], vec![], vec![]];

        let mut targets = vec![];

        for o in 0..gateway.len() {
            let g_octet = gateway[o];
            let s_octet = subnet[o];

            for ip in 0..=u8::MAX {
                if ip & s_octet == g_octet & s_octet {
                    octet_matches[o].push(ip);
                }
            }
        }

        for o0 in octet_matches[0].iter() {
            for o1 in octet_matches[1].iter() {
                for o2 in octet_matches[2].iter() {
                    for o3 in octet_matches[3].iter() {
                        targets.push(Ipv4Addr::new(*o0, *o1, *o2, *o3));
                    }
                }
            }
        }

        let target_count = targets.len() * port_count;
        *self.target_count.write().await = target_count;

        let (tx, rx) = flume::unbounded();
        let semaphore = Arc::new(Semaphore::new(concurrent_attempts as usize));

        for t in targets.iter() {
            for p in port_range.0..=port_range.1 {
                tokio::spawn(Self::attempt_connect(
                    tx.clone(),
                    *t,
                    p,
                    self.clone(),
                    semaphore.clone(),
                ));
            }
        }

        drop(tx);

        *self.semaphore.lock().await = Some(semaphore);
        while let Ok(stream) = rx.recv_async().await {
            if let Ok(a) = stream.peer_addr() {
                self.connects.write().await.push(a);
            }
            if let Some(h) = &(*self.handler.read().await) {
                h.handle(stream).await;
            }
        }

        *self.target_count.write().await = target_count;

        *self.state.write().await = ScannerState::Complete;
    }

    async fn attempt_connect(
        tx: flume::Sender<TcpStream>,
        ip: Ipv4Addr,
        port: u16,
        scanner: Arc<Self>,
        semaphore: Arc<Semaphore>,
    ) {
        let aquire = semaphore.acquire().await;
        if let Err(_) = aquire {
            *scanner.target_count.write().await -= 1;
            return;
        }
        let addr = (ip, port);
        if let Ok(s) = TcpStream::connect(addr).await {
            let _ = tx.send_async(s).await;
        }
        *scanner.target_count.write().await -= 1;
    }
}

impl GuiElement for Arc<Scanner> {
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        self.open.blocking_write()
    }

    fn name(&self) -> String {
        format!("Ip Scanner")
    }

    fn render(&self, _ctx: &egui::Context, ui: &mut Ui) {
        self.ui(ui);
    }
}
