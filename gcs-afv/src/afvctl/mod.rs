use std::sync::Arc;

use async_trait::async_trait;
use eframe::egui::{self, Ui, Window};
use tokio::{
    net::TcpStream,
    runtime::Runtime,
    sync::{Mutex, RwLock},
};

use crate::{
    afv::Afv,
    gui::GuiElement,
    network::{ComEngine, NetworkLogger, AFVPORT},
    scanner::{Scanner, ScannerAddrHandler},
};

pub struct AfvController {
    rt: Arc<Runtime>,
    open: RwLock<bool>,
    scanner: Mutex<Option<Arc<Scanner>>>,
    afv_links: RwLock<Vec<Arc<Afv>>>,
    dummy: RwLock<Option<Arc<Afv>>>,
}

impl AfvController {
    pub fn new(rt: Option<Arc<Runtime>>) -> Arc<AfvController> {
        let rt = match rt {
            Some(r) => r,
            None => Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Could not build runtime"),
            ),
        };
        Arc::new(Self {
            open: RwLock::new(false),
            scanner: Mutex::new(None),
            afv_links: RwLock::new(vec![]),
            rt,
            dummy: RwLock::new(None),
        })
    }
    fn side_panel(self: &Arc<Self>, ctx: &egui::Context, ui: &mut Ui) {
        egui::SidePanel::left("Conroller Contents")
            .resizable(true)
            .default_width(ui.available_size().x / 5.0)
            .show_inside(ui, |ui| {
                self.scanner_ui(ctx, ui);
                for _ in self.afv_links.blocking_read().iter() {
                    ui.label("AFV#");
                }
            });
    }
    fn scanner_ui(self: &Arc<Self>, ctx: &egui::Context, ui: &mut Ui) {
        let mut scanner_lock = self.scanner.blocking_lock();
        let scanner = scanner_lock.get_or_insert(self.clone().create_scanner());
        let mut open = scanner.open();
        ui.toggle_value(&mut open, "Open Scanner");
        if *open {
            Window::new("Scanner")
                .default_width(ui.available_size().x / 3.0)
                .resizable(true)
                .vscroll(true)
                .open(&mut open)
                .show(ctx, |ui| {
                    scanner.render(ctx, ui);
                });
        }
    }
    fn create_scanner(self: Arc<Self>) -> Arc<Scanner> {
        let scanner = Scanner::new(Some(self.rt.clone()));
        scanner.set_handler_blocking(self);
        scanner
    }
    pub fn spawn_dummy(self: &Arc<Self>) {
        self.rt.spawn(self.clone().dummy());
    }
    async fn dummy(self: Arc<Self>) {
        let afv = Afv::dummy(format!("127.0.0.1:{}", AFVPORT)).await;
        *self.dummy.write().await = Some(afv);
    }
}

#[async_trait]
impl ScannerAddrHandler for AfvController {
    async fn handle(&self, stream: TcpStream) {
        let com = ComEngine::afv_com_stream(stream);
        println!("Establishing connection with afv at {}", com.peer_addr());
        NetworkLogger::afv_com_monitor(&com).await;
        self.afv_links.write().await.push(Afv::link(com));
    }
}

impl GuiElement for Arc<AfvController> {
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        self.open.blocking_write()
    }

    fn name(&self) -> String {
        "Afv Controller".into()
    }

    fn render(&self, ctx: &egui::Context, ui: &mut eframe::egui::Ui) {
        self.side_panel(ctx, ui);
    }
}
