use std::sync::Arc;

use async_trait::async_trait;
use default_net::{Interface, get_interfaces};
use eframe::egui;
use tokio::{sync::RwLock, net::TcpStream};

use crate::gui::GuiElement;

#[async_trait]
pub trait ScannerHandler: Send + Sync{
    async fn handle(self: Arc<Self>, stream: TcpStream);
}

pub struct Scanner{
    open: RwLock<bool>,
    interfaces: Vec<Interface>,
    handler: RwLock<Option<Arc<dyn ScannerHandler>>>,
}

impl Scanner{
    pub fn new() -> Arc<Scanner> {
        let interfaces = get_interfaces();

        Arc::new(Self{
            open: RwLock::new(false),
            interfaces,
            handler: RwLock::new(None),
        })
        
    }
    pub async fn set_handler(&self, handler: Arc<dyn ScannerHandler>){
        *self.handler.write().await = Some(handler);
    }
    pub fn set_handler_blocking(&self, handler: Arc<dyn ScannerHandler>){
        *self.handler.blocking_write() = Some(handler);
    }
}

impl GuiElement for Arc<Scanner>{
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        self.open.blocking_write()
    }

    fn name(&self) -> String {
        "Scanner".into()
    }

    fn render(&self, ui: &mut eframe::egui::Ui) {
        let mut open = self.open();
        egui::Window::new("Scanner window")
            .open(&mut open)
        .show(ui.ctx(), |ui|{
            ui.label("Scanner");
                
        });
        
    }
}