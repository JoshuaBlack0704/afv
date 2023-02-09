use std::sync::Arc;

use tokio::{net::ToSocketAddrs, sync::RwLock};

use crate::{network::{ComEngine, AfvMessage, NetworkLogger}, gui::GuiElement};

use self::flir::Flir;

pub mod flir;

#[allow(unused)]
pub struct Afv{
    open: RwLock<bool>,
    com: Arc<ComEngine<AfvMessage>>,
    a50: Arc<Flir>,
}

impl Afv{
    pub async fn new(com: Arc<ComEngine<AfvMessage>>) -> Arc<Self> {
        let a50 = Flir::actuated(Some(com.clone())).await;
        NetworkLogger::afv_com_monitor(&com).await;
        Arc::new(
            Self{
                com,
                a50,
                open: RwLock::new(false),
            }
        )
    }
    pub async fn link(com: Arc<ComEngine<AfvMessage>>) -> Arc<Afv>{
        let a50 = Flir::linked(com.clone()).await;
        // NetworkLogger::afv_com_monitor(&com).await;
        Arc::new(
            Self{
                com,
                a50,
                open: RwLock::new(false),
            }
        )
    }

    pub async fn dummy(addr: impl ToSocketAddrs) -> Arc<Afv> {
        let com = ComEngine::afv_com_listen(addr).await.expect("Dummy afv could not establish connection");
        let a50 = Flir::actuated(Some(com.clone())).await;
        NetworkLogger::afv_com_monitor(&com).await;
        Arc::new(Self{
            com,
            a50,
            open: RwLock::new(false),
        })
    }
}

impl GuiElement for Arc<Afv>{
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        self.a50.open()
    }

    fn name(&self) -> String {
        "Afv".into()
    }

    fn render(&self, ui: &mut eframe::egui::Ui) {
        self.a50.render(ui);
    }
}


