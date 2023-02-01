use std::sync::Arc;

use clap::Parser;
use common_std::gndgui::{GuiElement, Tutorial};
use eframe::egui::{self, ScrollArea};
use flir::{RtspStream, A50, SampleImage};
use tokio::sync::{RwLock, Mutex};

pub type GuiElmType = Arc<dyn GuiElement>;

#[derive(Parser, Debug)]
pub struct Args{

    /// Set the target flir RTSP stream through its ip
    #[arg(long, default_value_t=String::new())]
    flir_ip: String,
}

impl Default for Args{
    fn default() -> Self {
        Self{
            flir_ip: String::from(""),
        }
    }
}

#[derive(Default)]
pub struct Terminal {
    empty: Mutex<bool>,
    elements: RwLock<Vec<GuiElmType>>,
    args: Args,
}

impl Terminal {
    pub fn new(_cc: &eframe::CreationContext<'_>, args: Args) -> Self {
        Self{
            elements: RwLock::new(vec![]),
            empty: Mutex::new(true),
            args,
        }
    }
    fn add_elements(&self){
        let mut elements = self.elements.blocking_write();
        let tutorial = Arc::new(Tutorial::new(String::from("Tutorial")));
        if self.args.flir_ip == ""{
            let flir_source = SampleImage::new(String::from("sample-fire.jpg"));
            let flir = Arc::new(A50::new(flir_source, None));
            A50::refresh_interval(flir.clone(), tokio::time::Duration::from_millis(16));
            elements.push(tutorial);
            elements.push(flir);
        }
        else{
            let flir_source = RtspStream::new(self.args.flir_ip.clone(), 0);
            let flir = Arc::new(A50::new(flir_source, None));
            A50::refresh_interval(flir.clone(), tokio::time::Duration::from_millis(16));
            elements.push(tutorial);
            elements.push(flir);
        }
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
        ctx.request_repaint();
        
   }
}



fn main() {
    let args = Args::parse();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("My egui App", native_options, Box::new(|cc| Box::new(Terminal::new(cc, args))));
}

