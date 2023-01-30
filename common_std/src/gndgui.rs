use std::sync::Arc;

use eframe::egui::{Ui, self, ScrollArea, Window, CentralPanel};
use tokio::sync::RwLock;

pub trait GuiElement{
    fn name(&self) -> String;
    fn render(&self, ctx: &egui::Context, frame: &mut eframe::Frame); 
    fn is_open(&self) -> bool;
    fn set_open(&self, status: bool);
}

pub struct Tutorial{
    name: String,
    is_open: RwLock<bool>,
}

impl Tutorial{
    pub fn new(name: String) -> Tutorial {
        Self{
            name,
            is_open: RwLock::new(false),
        }
    }
    fn ui(&self, ui: &mut Ui){
        ui.label("Tutorial!");
    }
}

impl Default for Tutorial{
    fn default() -> Self {
        Self{
            name: String::from("Tutorial"),
            is_open: RwLock::new(false),
        }
    }
}

impl GuiElement for Tutorial{
    fn name(&self) -> String {
        self.name.clone()
    }

    fn render(&self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut open = true;
        Window::new(self.name.clone())
        .open(&mut open)
        .constrain(true)
        .show(ctx, |ui| self.ui(ui));
        self.set_open(open);
    }

    fn is_open(&self) -> bool {
        *self.is_open.blocking_read()
    }

    fn set_open(&self, status: bool) {
        *self.is_open.blocking_write() = status;
    }
}
