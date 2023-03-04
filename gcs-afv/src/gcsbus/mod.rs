use std::sync::Arc;


use clap::Parser;
use eframe::{
    egui::{self, CentralPanel, Ui},
    App,
};

use tokio::{
    runtime::Handle,
    sync::Mutex, time::Duration,
};

use crate::{
    bus::Bus,
    AfvCtlMessage,
};

use self::bridgefinder::BridgeFinder;

pub mod bridgefinder;


pub trait Renderable {
    fn render(&self, ui: &mut Ui);
}

#[derive(Parser, Debug)]
pub struct GcsArgs{
    
}

pub struct Gcs {
    _handle: Handle,
    ui_target: Mutex<Arc<dyn Renderable>>,

    bridge_finder: Arc<BridgeFinder>,
}

impl Gcs {
    pub fn launch() {
        let _args = GcsArgs::parse();
        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Could not build async runtime"),
        );

        let gcs = rt.block_on(Self::new());

        let opts = eframe::NativeOptions::default();
        eframe::run_native("Ground Control", opts, Box::new(|cc| gcs.build(cc)));
    }

    fn build(self, _cc: &eframe::CreationContext<'_>) -> Box<Self> {
        Box::new(self)
    }
    fn top_panel(&self, ui: &mut Ui) {
        egui::TopBottomPanel::top("Title").show_inside(ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label("Ground Control Station");
            });
            ui.horizontal(|ui| {
                if ui.button("Afv Poller").clicked() {
                    *self.ui_target.blocking_lock() = self.bridge_finder.clone();
                }
            });
        });
    }
    fn central_panel(&self, ui: &mut Ui) {
        CentralPanel::default().show_inside(ui, |ui| {
            self.ui_target.blocking_lock().render(ui);
        });
    }
    async fn new() -> Gcs {
        let bus = Bus::<AfvCtlMessage>::new().await;
        let poller = BridgeFinder::new(bus.clone(), Duration::from_secs(2)).await;
        bus.add_element(poller.clone()).await;
        Self {
            _handle: Handle::current(),
            bridge_finder: poller.clone(),
            ui_target: Mutex::new(poller.clone()),
        }
    }
}

impl App for Gcs {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.bridge_finder.process_bridges();
        CentralPanel::default().show(ctx, |ui| {
            self.top_panel(ui);
            self.central_panel(ui);
        });
    }
}
