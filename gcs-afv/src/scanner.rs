use std::{sync::Arc, net::{Ipv4Addr, SocketAddr}};

use async_trait::async_trait;
use default_net::{Interface, get_interfaces, interface::InterfaceType};
use eframe::egui::{self, Ui, DragValue};
use ipnet::Ipv4Net;
use tokio::{sync::{RwLock, Semaphore}, net::{TcpStream, TcpSocket}, runtime::{Runtime, Handle}};

use crate::gui::GuiElement;

#[async_trait]
pub trait ScannerHandler: Send + Sync{
    async fn handle(self: Arc<Self>, stream: TcpStream);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State{
    Selection,
    Scanning,
    Results,
}

pub struct Scanner{
    handle: Handle,
    open: RwLock<bool>,
    state: RwLock<State>,
    interfaces: Vec<Interface>,
    tgt_net: RwLock<(String, Interface)>,
    ports: RwLock<Vec<u16>>,
    handler: RwLock<Option<Arc<dyn ScannerHandler>>>,
    scan_status: RwLock<(usize, Vec<SocketAddr>)>,
    semaphore: RwLock<Arc<Semaphore>>,
    input_port: RwLock<u16>,
}

impl Scanner{
    pub async fn new(interface_filter: Option<InterfaceType>) -> Arc<Scanner> {
        let mut interfaces = get_interfaces();
        if let Some(f) = interface_filter{
            interfaces = interfaces.iter().filter(|i| i.if_type == f).map(|f|f.clone()).collect();
        }
        let net = Self::interface_to_net(interfaces[0].clone());
        let semaphore = Semaphore::new(1000);

        Arc::new(Self{
            open: RwLock::new(false),
            interfaces: interfaces.clone(),
            handler: RwLock::new(None),
            ports: Default::default(),
            tgt_net: RwLock::new((net.0, interfaces[0].clone())),
            state: RwLock::new(State::Selection),
            handle: Handle::current(),
            scan_status: Default::default(),
            semaphore: RwLock::new(Arc::new(semaphore)),
            input_port: Default::default(),
        })
        
    }
    pub fn new_blocking(rt: Arc<Runtime>, interface_filter: Option<InterfaceType>) -> Arc<Scanner> {
        rt.block_on(Self::new(interface_filter))
    }
    fn interface_to_net(interface: Interface) -> (String, Ipv4Net) {
        let ip = interface.ipv4[0].clone();
        let name = match interface.friendly_name{
            Some(n) => n,
            None => interface.name,
        };
        (name, Ipv4Net::with_netmask(ip.addr, ip.netmask).expect(&format!("Could not generate ip net from ip {:?} and net {:?}", ip.addr, ip.netmask)))
    }
    pub async fn add_port(&self, port: u16){
        let mut ports = self.ports.write().await;

        if !ports.contains(&port){
            ports.push(port);
        }
    }
    pub fn add_port_blocking(&self, port: u16){
        let mut ports = self.ports.blocking_write();

        if !ports.contains(&port){
            ports.push(port);
        }
    }
    pub async fn set_handler(&self, handler: Arc<dyn ScannerHandler>){
        *self.handler.write().await = Some(handler);
    }
    pub fn set_handler_blocking(&self, handler: Arc<dyn ScannerHandler>){
        *self.handler.blocking_write() = Some(handler);
    }
    fn selection_ui(self: &Arc<Self>, ui: &mut Ui){
        let mut tgt = self.tgt_net.blocking_write();

        ui.vertical_centered_justified(|ui|{
            egui::containers::ComboBox::from_label("Select Interface")
            .selected_text(tgt.0.clone())
            .show_ui(ui, |ui| {
                for interface in self.interfaces.iter(){
                    let net = Scanner::interface_to_net(interface.clone());
                    if ui.selectable_value(&mut tgt.0, net.0.clone(), net.0.clone()).clicked(){
                       *tgt = (net.0, interface.clone()); 
                    }
                }
            });
            ui.separator();
            let net = Self::interface_to_net(tgt.1.clone());
            ui.label(format!("Network: {}", net.1)); 
            ui.label("Target Ports:");
            ui.horizontal(|ui|{
                for p in self.ports.blocking_read().iter(){
                    ui.label(format!("{}", p));
                }
            });
            ui.separator();
            ui.horizontal(|ui|{
                let mut port = self.input_port.blocking_write();
                let drag = DragValue::new(&mut (*port)).clamp_range(0..=u16::MAX);
                ui.add(drag);
                if ui.button("Add port").clicked(){
                    let mut ports = self.ports.blocking_write();
                    if !ports.contains(&port){
                        ports.push(*port);
                    }
                }
                if ui.button("Clear ports").clicked(){
                    self.ports.blocking_write().clear();
                }
            });
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Parallel scan attempts: ");
                let mut semaphore = self.semaphore.blocking_write();
                let mut permits = semaphore.available_permits();
                let drag = egui::widgets::DragValue::new(&mut permits).clamp_range(1..=10000);
                ui.add(drag);
                if permits != semaphore.available_permits(){
                    *semaphore = Arc::new(Semaphore::new(permits));
                }
            });
            if ui.button("Scan").clicked(){
                self.clone().dispatch();
            }
            if ui.button("Scan all interfaces").clicked(){
                self.clone().dispatch_all_interfaces();
            }
        });
    }
    fn scanning_ui(&self, ui: &mut Ui){
        ui.vertical_centered_justified(|ui|{
            ui.label("Scan in progress");
            if ui.button("Cancel scan").clicked(){
                self.semaphore.blocking_read().close();
            }
            ui.separator();
            let status = self.scan_status.blocking_read().clone();
            ui.label(format!("Scans waiting: {}, Successful connects: ", status.0));
            for addr in status.1.iter(){
                ui.label(format!("{}", addr));
            }
        });
        
    }
    fn results_ui(&self, ui: &mut Ui){
        ui.vertical_centered_justified(|ui|{
            ui.label("Scan Results");
            if ui.button("Restart").clicked(){
                *self.state.blocking_write() = State::Selection;
            }
            ui.separator();
            let status = self.scan_status.blocking_read().clone();
            ui.label("Successful connects: ");
            for addr in status.1.iter(){
                ui.label(format!("{}", addr));
            }
        });
        
        
    }

    pub async fn scan_interfaces(self: Arc<Self>, interfaces: Vec<Interface>){
        if *self.state.read().await == State::Scanning{
            println!("Scan in progress, aborting");
            return;
        }
        println!("Scan started for interfaces: {:?}", interfaces.iter().map(|i| Self::interface_to_net(i.clone()).0).collect::<Vec<String>>());
        *self.state.write().await = State::Scanning;
        let mut addresses = 0;
        let ports = self.ports.read().await.clone();
        let responses:(flume::Sender<Option<SocketAddr>>, flume::Receiver<Option<SocketAddr>>) = flume::unbounded();

        for interface in interfaces.iter(){
            let net = Self::interface_to_net(interface.clone());
            let interface_addrs:Vec<Ipv4Addr>;
            if net.1.addr().is_loopback(){
                interface_addrs = vec![net.1.addr()];
                
            }
            else{
                interface_addrs = net.1.hosts().collect();
            }
            
            for addr in interface_addrs.iter(){
                for port in ports.iter(){
                    addresses += 1;
                    let tx = responses.0.clone();
                    let scanner = self.clone();
                    let addr = addr.clone();
                    let port = port.clone();
        
                    tokio::spawn(async move {
                        let semaphore = scanner.semaphore.read().await.clone();
                        if let Err(_) = semaphore.acquire().await{
                            let _ = tx.send(None);
                            return;
                        }
                        if addr.is_loopback(){
                            if let Ok(s) = TcpStream::connect((addr, port)).await{
                                let _ = tx.send(Some(s.peer_addr().expect("Could not get peer addr")));
                                if let Some(h) = & (*scanner.handler.read().await){
                                    tokio::spawn(h.clone().handle(s));
                                }
                                return;
                            }
                        }
                        else{
                            let socket = TcpSocket::new_v4().unwrap();
                            if let Err(_) = socket.bind((net.1.addr(), 0u16).into()){
                                let _ = tx.send(None);
                                return;
                            }
                            if let Ok(s) = socket.connect((addr, port).into()).await{
                                let _ = tx.send(Some(s.peer_addr().expect("Could not get peer addr")));
                                if let Some(h) = & (*scanner.handler.read().await){
                                    tokio::spawn(h.clone().handle(s));
                                }
                                return;
                            }
                        }
                        let _ = tx.send(None);
                    });
                }
            }
            
        }

        drop(responses.0);

        *self.scan_status.write().await = (addresses, vec![]);
        while let Ok(r) = responses.1.recv_async().await{
            match r{
                Some(addr) => {
                    let mut status = self.scan_status.write().await;
                    status.0 -= 1;
                    status.1.push(addr);
                },
                None => self.scan_status.write().await.0 -= 1,
            }
        }
        println!("Scan for interfaces {:?} stopped", interfaces.iter().map(|i| Self::interface_to_net(i.clone()).0).collect::<Vec<String>>());
        *self.state.write().await = State::Results;
    }
    pub fn dispatch(self: Arc<Self>){
        self.clone().handle.spawn(async move{
            self.clone().scan_interfaces(vec![self.tgt_net.read().await.1.clone()]).await
        });
    }
    pub fn dispatch_all_interfaces(self: Arc<Self>){
        self.clone().handle.spawn(async move{
            self.clone().scan_interfaces(self.interfaces.clone()).await
        });
    }
}

impl GuiElement for Scanner{
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        self.open.blocking_write()
    }

    fn name(&self) -> String {
        "Scanner".into()
    }

    fn render(self: Arc<Self>, ui: &mut eframe::egui::Ui) {
        let state = *self.state.blocking_read();
        match state{
            State::Selection => self.selection_ui(ui),
            State::Scanning => self.scanning_ui(ui),
            State::Results => self.results_ui(ui),
        };
    }
}