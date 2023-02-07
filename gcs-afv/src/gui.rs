use std::sync::Arc;
use eframe::{self, egui::{self, Ui}};
use tokio::sync::{RwLockWriteGuard, RwLock};

pub trait GuiElement{
    fn open(&self) -> RwLockWriteGuard<bool>;
    fn name(&self) -> String;
    fn render(&self, ui: &mut Ui);
}

pub trait GuiArgs{
    
}

pub struct Terminal{
    elements: Vec<Arc<dyn GuiElement>>,
    continuous_refresh: RwLock<bool>,
}

pub struct TerminalBuilder{
    elements: Vec<Arc<dyn GuiElement>>,
}

pub struct Tutorial{
    open: RwLock<bool>,
}

impl GuiElement for Tutorial{
    fn open(&self) -> RwLockWriteGuard<bool> {
        self.open.blocking_write()
    }

    fn name(&self) -> String {
        format!("Tutorial")
    }

    fn render(&self, ui: &mut Ui) {
        ui.label("Test");
    }
}

impl Tutorial{
    pub fn new() -> Arc<Tutorial> {
        Arc::new(
            Self{
                open: RwLock::new(false),
            }
        )
    }
}

impl TerminalBuilder{
    pub fn new() -> TerminalBuilder {
        Self{
            elements: vec![],
        }
    }
    pub fn add_element(mut self, element: Arc<dyn GuiElement>) -> TerminalBuilder {
        self.elements.push(element);
        self
    }
    pub fn build(self, _cc: &eframe::CreationContext<'_>, _args: Arc<impl GuiArgs>) -> Box<Terminal> {

        Box::new(Terminal{
            elements: self.elements,
            continuous_refresh: RwLock::new(true),
        })
        
    }
    pub fn launch(self, args: &Arc<impl GuiArgs + 'static>){
        let args = args.clone();
        let opts = eframe::NativeOptions::default();
        eframe::run_native("Ground Control", opts, Box::new(|cc| self.build(cc, args)));
    }
}

impl Terminal{
    fn side_panel(&self, ctx: &egui::Context){
        let list_elements = |ui: &mut Ui| {
            let mut refresh = self.refresh();
            ui.toggle_value(&mut refresh, "Continouous refersh");
            for e in self.elements.iter(){
                let mut open = e.open();
                ui.toggle_value(&mut open, e.name());
            }
        };
        
        egui::SidePanel::left("Elements")
        .resizable(false)
        .default_width(150.0)
        .show(ctx, |ui|{
           ui.vertical_centered(|ui| {
                list_elements(ui);
            }) 
        });
    }

    fn central_panel(&self, ctx: &egui::Context){
        let display_elements = |ui: &mut Ui|{
            for e in self.elements.iter(){
                if *e.open(){
                    e.render(ui);
                    break;
                }
            }
        };

        egui::CentralPanel::default().show(ctx, display_elements);
    }
    fn refresh(&self) -> RwLockWriteGuard<bool> {
        self.continuous_refresh.blocking_write()
    }
}

impl eframe::App for Terminal{
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.side_panel(ctx);
        self.central_panel(ctx);
        if *self.refresh(){
            ctx.request_repaint();
        }
    }
    
}