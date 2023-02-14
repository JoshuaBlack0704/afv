use std::sync::Arc;

use eframe::egui::ComboBox;
use tokio::{net::ToSocketAddrs, sync::RwLock};

use crate::{network::{ComEngine, AfvMessage, NetworkLogger}, gui::GuiElement};

use self::{flir::Flir, turret::Turret};

pub mod flir;
pub mod turret;
pub mod turretv2;

#[derive(Debug, PartialEq, Eq)]
pub enum GuiSystem{
    Flir,
    Turret,
}

#[allow(unused)]
pub struct Afv{
    open: RwLock<bool>,
    com: Arc<ComEngine<AfvMessage>>,
    gui_system: RwLock<GuiSystem>,
    a50: Arc<Flir>,
    turret: Arc<Turret>,
}

impl Afv{
    pub async fn new(com: Arc<ComEngine<AfvMessage>>) -> Arc<Self> {
        let a50 = Flir::actuated(Some(com.clone())).await;
        let turret = Turret::actuated(Some(com.clone()), a50.clone()).await;
        NetworkLogger::afv_com_monitor(&com).await;
        Arc::new(
            Self{
                com,
                a50,
                open: RwLock::new(false),
                gui_system: RwLock::new(GuiSystem::Flir),
                turret,
            }
        )
    }
    pub async fn link(com: Arc<ComEngine<AfvMessage>>) -> Arc<Afv>{
        let a50 = Flir::linked(com.clone()).await;
        let turret = Turret::linked(com.clone(), a50.clone()).await;
        // NetworkLogger::afv_com_monitor(&com).await;
        Arc::new(
            Self{
                com,
                a50,
                open: RwLock::new(false),
                gui_system: RwLock::new(GuiSystem::Flir),
                turret,
            }
        )
    }

    pub async fn simulated(addr: impl ToSocketAddrs) -> Arc<Afv> {
        let com = ComEngine::afv_com_listen(addr).await.expect("Dummy afv could not establish connection");
        let a50 = Flir::actuated(Some(com.clone())).await;
        let turret = Turret::simulated(Some(com.clone()), a50.clone()).await;
        NetworkLogger::afv_com_monitor(&com).await;
        Arc::new(Self{
            com,
            a50,
            open: RwLock::new(false),
            gui_system: RwLock::new(GuiSystem::Flir),
            turret,
        })
    }
}

impl GuiElement for Afv{
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        self.a50.open()
    }

    fn name(&self) -> String {
        "Afv".into()
    }

    fn render(self: Arc<Self>, ui: &mut eframe::egui::Ui) {
        let mut selected = self.gui_system.blocking_write();
        ComboBox::from_label("Available Systems")
            .selected_text(format!("{:?}", *selected))
            .show_ui(ui, |ui|{
                ui.selectable_value(&mut (*selected), GuiSystem::Flir, "Flir");
                ui.selectable_value(&mut (*selected), GuiSystem::Turret, "Turret");
                
            });
        ui.separator();
        match *selected{
            GuiSystem::Flir => {
                self.a50.clone().render(ui);
            },
            GuiSystem::Turret => {
                self.turret.clone().render(ui);
            },
        }
    }
}


