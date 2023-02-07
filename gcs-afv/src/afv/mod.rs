use std::sync::Arc;

use tokio::{net::ToSocketAddrs, runtime::Runtime, sync::RwLock};

use crate::{network::{ComEngine, AfvMessage, NetworkLogger}, gui::GuiElement};

use self::flir::{A50, RtspSession, A50Link};

pub mod flir;

#[allow(unused)]
pub struct Afv{
    open: RwLock<bool>,
    com: Arc<ComEngine<AfvMessage>>,
    a50: Arc<A50>,
}

impl Afv{
    pub async fn new(rt: Option<Arc<Runtime>>, com: Arc<ComEngine<AfvMessage>>) -> Arc<Self> {
        let rt = match rt {
            Some(r) => r,
            None => Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Could not build runtime"),
            ),
        };
        let rtsp = RtspSession::new_blocking(rt.clone());
        let a50 = A50::new(Some(rt.clone()), Arc::new(rtsp.clone()));
        com.add_listener(a50.clone()).await;
        NetworkLogger::afv_com_monitor(&com).await;
        Arc::new(
            Self{
                com,
                a50,
                open: RwLock::new(false),
            }
        )
    }
    pub async fn link(rt: Option<Arc<Runtime>>, com: Arc<ComEngine<AfvMessage>>) -> Arc<Afv>{
        let rt = match rt {
            Some(r) => r,
            None => Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Could not build runtime"),
            ),
        };
        let link = A50Link::new(com.clone()).await;
        let a50 = A50::new(Some(rt.clone()), Arc::new(link.clone()));
        a50.clone().refresh_interval(tokio::time::Duration::from_millis(1000));
        // NetworkLogger::afv_com_monitor(&com).await;
        Arc::new(
            Self{
                com,
                a50,
                open: RwLock::new(false),
            }
        )
    }

    pub async fn dummy(rt: Option<Arc<Runtime>>, addr: impl ToSocketAddrs) -> Arc<Afv> {
        let rt = match rt {
            Some(r) => r,
            None => Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Could not build runtime"),
            ),
        };
        let com = ComEngine::afv_com_listen(addr).await.expect("Dummy afv could not establish connection");
        let rtsp = RtspSession::new(Some(rt.clone())).await;
        let a50 = A50::new(Some(rt.clone()), Arc::new(rtsp.clone()));
        com.add_listener(a50.clone()).await;
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
        self.open.blocking_write()
    }

    fn name(&self) -> String {
        "Afv".into()
    }

    fn render(&self, ui: &mut eframe::egui::Ui) {
        self.a50.render(ui);
    }
}


