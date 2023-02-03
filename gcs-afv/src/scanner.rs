use std::{net::Ipv4Addr, sync::Arc};

use eframe::egui::{Ui, self};
use tokio::{sync::{RwLock, Mutex}, runtime::Runtime};

use crate::{gui::GuiElement, network::AFVPORT};

#[derive(Clone, Copy)]
pub enum ScannerState{
    Available,
    Dispatched,
    Complete,
}

pub struct Scanner{
    rt: Arc<Runtime>,
    gateway: Mutex<Ipv4Addr>,
    subnet: Mutex<Ipv4Addr>,
    port_range: Mutex<(u16, u16)>,
    state: RwLock<ScannerState>,
    open: RwLock<bool>,
}

impl Scanner{
    pub fn new(rt: Option<Arc<Runtime>>) -> Arc<Scanner> {
        let rt = match rt{
            Some(rt) => rt,
            None => Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not construct runtime for scanenr")),
        };
        
        Arc::new(
            Self{
                gateway: Mutex::new(Ipv4Addr::new(192,196,1,0)),
                subnet: Mutex::new(Ipv4Addr::new(255,255,255,0)),
                port_range: Mutex::new((AFVPORT, AFVPORT)),
                state: RwLock::new(ScannerState::Available),
                rt,
                open: RwLock::new(false),
            }
        )
    }
    pub fn ui(self: &Arc<Self>, ui: &mut Ui){
        let state = *self.state.blocking_read();

        match state{
            ScannerState::Available => {
                self.available_ui(ui);
            },
            ScannerState::Dispatched => {
                self.dispatched_ui(ui);
                
            },
            ScannerState::Complete => {
                self.completed_ui(ui);
                
            },
        }
    }
    fn available_ui(self: &Arc<Self>, ui: &mut Ui){
        let mut state = self.state.blocking_write();
        let mut gateway = self.gateway.blocking_lock();
        let mut subnet = self.subnet.blocking_lock();
        let mut port_range = self.port_range.blocking_lock();

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
                    let drag_val = egui::DragValue::new(&mut port_range.0).clamp_range(0..=u16::MAX);
                    ui.add(drag_val);
                    let drag_val = egui::DragValue::new(&mut port_range.1).clamp_range(0..=u16::MAX);
                    ui.add(drag_val);
                });
        
        
            let port_count:u64 = (port_range.0..=port_range.1).count() as u64;
            let mut subnet = u32::from_be_bytes(s_octets);
            let mut subnet_bits:u32 = 0;
            
            for _ in 0..u32::BITS{
                if subnet & 1 == 0{
                    subnet_bits += 1;
                }
                subnet >>= 1;
            }

            let count = port_count * 2u64.pow(subnet_bits);
            let count = format!("Total targets: {}", count);
        
            ui.label(count);
            if ui.button("Start").clicked(){
                *state = ScannerState::Dispatched;
                self.rt.spawn(self.clone().dispatch());
            }; 
        });
    }
    fn dispatched_ui(self: &Arc<Self>, ui: &mut Ui){
        
    }
    fn completed_ui(self: &Arc<Self>, ui: &mut Ui){
        
    }
    async fn dispatch(self: Arc<Self>){
        
    }
}

impl GuiElement for Arc<Scanner>{
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        self.open.blocking_write()
    }

    fn name(&self) -> String {
        format!("Ip Scanner")
    }

    fn render(&self, ui: &mut Ui) {
        self.ui(ui);
    }
}