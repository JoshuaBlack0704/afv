use std::sync::Arc;

use clap::Parser;

use eframe::{
    egui::{self, CentralPanel, TopBottomPanel, Ui},
    CreationContext,
};
use tokio::{runtime::Runtime, sync::Mutex};

use crate::{
    communicators::afv::AfvCommuncation, network::scanner::ScanCount, operators::afv_launcher,
};

/// Generic representation of a renderable object
pub trait Renderable {
    fn render(&mut self, ui: &mut Ui);
}

#[derive(Parser)]
/// Used with the clap crate to pass command line arguments to the process
struct GcsArgs {
    #[arg(short, long)]
    simulate: bool,
}

/// This is the main starting struct for the ground station
/// This struct is reponsible for creating the eframe event loop and opening the window
pub struct GcsUi {
    runtime: Runtime,
    connected_afvs: Arc<Mutex<Vec<AfvCommuncation>>>,
    selected_afv: u64,
}

impl GcsUi {
    /// Starts the eframe event loop
    pub fn launch() {
        eframe::run_native(
            "Afv Ground Control Station",
            Default::default(),
            Box::new(|cc| Self::run(cc)),
        );
    }
    pub fn run(_cc: &CreationContext) -> Box<GcsUi> {
        let args = GcsArgs::parse();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Could not start tokio runtime");

        if args.simulate {
            runtime.spawn(afv_launcher::simulate());
        }

        let connected_afvs = Arc::new(Mutex::new(vec![]));

        runtime.spawn(AfvCommuncation::find_afvs(
            connected_afvs.clone(),
            ScanCount::Limited(2),
        ));

        Box::new(Self {
            runtime,
            connected_afvs,
            selected_afv: 0,
        })
    }

    /// Renders the top panel
    fn top_panel(&mut self, ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.label("Afv Ground Station System");
        });
        ui.horizontal(|ui| {
            egui::ComboBox::from_label("Connected Afvs")
                .selected_text(format!("{:x}", self.selected_afv))
                .show_ui(ui, |ui| {
                    for afv in self.connected_afvs.blocking_lock().iter_mut() {
                        let uuid = self.runtime.block_on(afv.uuid());
                        ui.selectable_value(&mut self.selected_afv, uuid, format!("{:x}", uuid));
                    }
                })
        });
    }

    /// Renders the central panel, which is just the selected afv communicator
    fn central_panel(&mut self, ui: &mut Ui) {
        for afv in self.connected_afvs.blocking_lock().iter_mut() {
            if self.runtime.block_on(afv.uuid()) == self.selected_afv {
                afv.render(ui);
                break;
            }
        }
    }
}

impl eframe::App for GcsUi {
    /// This is the required function needed operate the eframe event loop
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("Title Section").show(ctx, |ui| {
            self.top_panel(ui);
        });
        CentralPanel::default().show(ctx, |ui| {
            self.central_panel(ui);
        });
        ctx.request_repaint();
    }
}
