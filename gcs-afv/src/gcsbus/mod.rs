#![allow(unused)]
use std::sync::Arc;

use async_trait::async_trait;
use eframe::{App, egui};
use rand::{thread_rng, Rng};
use tokio::runtime::Runtime;

use crate::{AfvCtlMessage, bus::{Bus, BusElement}, GcsArgs};

pub mod afvpoller;

pub struct Gcs{
    uuid: u64,
    runtime: Arc<Runtime>,
    bus: Bus<AfvCtlMessage>,
}

impl Gcs{
    pub fn launch(args: GcsArgs){
        let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not build async runtime"));
        let bus = Bus::<AfvCtlMessage>::new_blocking(&rt);

        // Add all bus elements

        let gcs = Self{
            runtime: rt,
            bus,
            uuid: thread_rng().gen::<u64>(),
        };

        
        let opts = eframe::NativeOptions::default();
        eframe::run_native("Ground Control", opts, Box::new(|cc| gcs.build(cc)));
    }

    fn build(self, _cc: &eframe::CreationContext<'_>) -> Box<Self>{
        Box::new(self)
    } 
}

impl App for Gcs{
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("Title").show(ctx, |ui|{});
    }
}

#[async_trait]
impl BusElement<AfvCtlMessage> for Gcs{
    async fn recieve(self: Arc<Self>, msg: AfvCtlMessage){}
    fn uuid(&self) -> u64{
        self.uuid
    }
}