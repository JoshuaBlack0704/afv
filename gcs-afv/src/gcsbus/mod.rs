#![allow(unused)]
use std::sync::Arc;

use async_trait::async_trait;
use eframe::{
    egui::{self, CentralPanel, Ui},
    App,
};
use rand::{thread_rng, Rng};
use tokio::{
    runtime::{Handle, Runtime},
    sync::Mutex,
};

use crate::{
    bus::{Bus, BusElement},
    AfvCtlMessage, GcsArgs,
};

use self::afvpoller::AfvPoller;

pub mod afvpoller;

pub trait Renderable {
    fn render(&self, ui: &mut Ui);
}

pub struct Gcs {
    uuid: u64,
    handle: Handle,
    bus: Bus<AfvCtlMessage>,
    ui_target: Mutex<Arc<dyn Renderable>>,

    poller: Arc<AfvPoller>,
}

impl Gcs {
    pub fn launch(args: GcsArgs) {
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
                    *self.ui_target.blocking_lock() = self.poller.clone();
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
        let poller = AfvPoller::new(bus.clone()).await;
        bus.add_element(poller.clone());
        Self {
            uuid: thread_rng().gen::<u64>(),
            handle: Handle::current(),
            bus,
            poller: poller.clone(),
            ui_target: Mutex::new(poller.clone()),
        }
    }
}

impl App for Gcs {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            self.top_panel(ui);
            self.central_panel(ui);
        });
    }
}
