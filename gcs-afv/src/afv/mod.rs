use std::sync::Arc;

use eframe::egui::ComboBox;
use tokio::{net::ToSocketAddrs, sync::RwLock};

use crate::{network::{ComEngine, AfvMessage, NetworkLogger}, gui::GuiElement};

use self::{flir::Flir, turret::Turret, mainctl::MainCtl};

pub mod flir;
pub mod turret;
pub mod mainctl;

#[derive(Debug, PartialEq, Eq)]
pub enum GuiSystem{
    Flir,
    Turret,
    MainCtl,
}

#[allow(unused)]
pub struct Afv{
    open: RwLock<bool>,
    com: Arc<ComEngine<AfvMessage>>,
    gui_system: RwLock<GuiSystem>,
    top_flir: Arc<Flir>,
    turret: Arc<Turret>,
    mainctl: Arc<MainCtl>,
}

impl Afv{
    pub async fn actuated(com: Arc<ComEngine<AfvMessage>>) -> Arc<Self> {
        let top_flir = Flir::actuated(Some(com.clone())).await;
        let turret = Turret::simulated(Some(com.clone()), top_flir.clone()).await;
        let mainctl = MainCtl::actuated(Some(com.clone())).await;
        NetworkLogger::afv_com_monitor(&com).await;
        Arc::new(
            Self{
                com,
                top_flir,
                open: RwLock::new(false),
                gui_system: RwLock::new(GuiSystem::Flir),
                turret,
                mainctl,
            }
        )
    }
    pub async fn link(com: Arc<ComEngine<AfvMessage>>) -> Arc<Afv>{
        let top_flir = Flir::linked(com.clone()).await;
        let turret = Turret::linked(com.clone(), top_flir.clone()).await;
        let mainctl = MainCtl::linked(com.clone()).await;
        // NetworkLogger::afv_com_monitor(&com).await;
        Arc::new(
            Self{
                com,
                top_flir,
                open: RwLock::new(false),
                gui_system: RwLock::new(GuiSystem::Flir),
                turret,
                mainctl,
            }
        )
    }

    pub async fn simulated(addr: impl ToSocketAddrs) -> Arc<Afv> {
        let com = ComEngine::afv_com_listen(addr).await.expect("Dummy afv could not establish connection");
        let top_flir = Flir::actuated(Some(com.clone())).await;
        let turret = Turret::simulated(Some(com.clone()), top_flir.clone()).await;
        // let mainctl = MainCtl::simulated(Some(com.clone())).await;
        let mainctl = MainCtl::actuated(Some(com.clone())).await;
        NetworkLogger::afv_com_monitor(&com).await;
        Arc::new(Self{
            com,
            top_flir,
            open: RwLock::new(false),
            gui_system: RwLock::new(GuiSystem::Flir),
            turret,
            mainctl,
        })
    }
}

impl GuiElement for Afv{
    fn open(&self) -> tokio::sync::RwLockWriteGuard<bool> {
        self.top_flir.open()
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
                ui.selectable_value(&mut (*selected), GuiSystem::MainCtl, "MainCtl");
            });
        ui.separator();
        match *selected{
            GuiSystem::Flir => {
                self.top_flir.clone().render(ui);
            },
            GuiSystem::Turret => {
                self.turret.clone().render(ui);
            },
            GuiSystem::MainCtl => {
                self.mainctl.clone().render(ui);
            },
        }
    }
}


