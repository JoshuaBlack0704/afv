use std::sync::Arc;

use common_std::gndgui::{GuiElement, Tutorial};
use eframe::egui::{self, ScrollArea};
use flir::{RtspStream, A50, SampleImage};
use tokio::sync::{RwLock, Mutex};

pub type GuiElmType = Arc<dyn GuiElement>;

#[derive(Default)]
pub struct Terminal {
    empty: Mutex<bool>,
    elements: RwLock<Vec<GuiElmType>>,
}

impl Terminal {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self{
            elements: RwLock::new(vec![]),
            empty: Mutex::new(true),
        }
    }
    fn add_elements(&self){
        let mut elements = self.elements.blocking_write();
        let tutorial = Arc::new(Tutorial::new(String::from("Tutorial")));
        // let flir_source = SampleImage::new(String::from("sample-fire.jpg"));
        let flir_source = RtspStream::new("10.192.138.49");
        // let flir_source = RtspStream::new_url("rtsp://ipvmdemo.dyndns.org:554/h264&basic_auth=");
        let flir = Arc::new(A50::new(flir_source, None));
        flir.update_image_blocking();
        elements.push(tutorial);
        elements.push(flir);
    }
}

impl eframe::App for Terminal {
   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut empty = self.empty.blocking_lock();
        if *empty{
            self.add_elements();
            *empty = false;
        }
        
        let elements = self.elements.blocking_read().clone();
        egui::SidePanel::left("Services")
            .resizable(false)
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| ui.heading("Services"));
                ui.separator();
                ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        for e in elements.iter(){
                            let mut open = e.is_open();
                            ui.toggle_value(&mut open, e.name());
                            e.set_open(open);
                            if open{
                                e.render(ctx, frame);
                            }
                        }
                        
                    });
                });
            });
        
   }
}



fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("My egui App", native_options, Box::new(|cc| Box::new(Terminal::new(cc))));
}

