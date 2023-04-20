use std::sync::Arc;

use eframe::{
    egui::{TextureHandle, Ui, DragValue},
    epaint::ColorImage,
};
use image::{DynamicImage, ImageBuffer};
use log::{debug, error};
use openh264::{decoder::Decoder, nal_units, to_bitstream_with_001_be};

use tokio::{
    sync::{broadcast, watch, Notify},
    time::{sleep, Duration},
};

use crate::{
    drivers::flir::FlirDriverMessage, network::NetMessage, operators::flir::{FlirOperatorSettings, FlirOperatorMessage, FlirAnalysis},
    ui::Renderable,
};

#[derive(Clone)]
pub struct FlirSystemCommunicator {
    net_tx: broadcast::Sender<NetMessage>,
    settings_watch: Arc<watch::Sender<FlirOperatorSettings>>,
    image_watch: Arc<watch::Sender<DynamicImage>>,
    image_analysis_watch: Arc<watch::Sender<FlirAnalysis>>,
    gui_image_watch: Arc<watch::Sender<ColorImage>>,
    stream_ir_request: Arc<Notify>,
    stream_visual_request: Arc<Notify>,

    //Ui parameters
    adjustable_settings: FlirOperatorSettings,
    stream_ir: bool,
    stream_visual: bool,
    gui_image_receiver: watch::Receiver<ColorImage>,
    settings_reciever: watch::Receiver<FlirOperatorSettings>,
    image_analysis_receiver: watch::Receiver<FlirAnalysis>,
    gui_texture: TextureHandle,
}

impl FlirSystemCommunicator {
    pub async fn new(net_tx: broadcast::Sender<NetMessage>, ui: &mut Ui) -> FlirSystemCommunicator {
        let gui_image_watch = watch::channel(ColorImage::example());
        let settings_watch = watch::channel(Default::default());
        let image_analysis_watch = watch::channel(Default::default());

        let comm = Self {
            net_tx,
            settings_watch: Arc::new(settings_watch.0),
            image_watch: Arc::new(watch::channel(Default::default()).0),
            gui_image_watch: Arc::new(gui_image_watch.0),
            stream_ir_request: Default::default(),
            adjustable_settings: Default::default(),
            stream_ir: false,
            stream_visual: false,
            stream_visual_request: Default::default(),
            gui_image_receiver: gui_image_watch.1.clone(),
            gui_texture: ui.ctx().load_texture(
                "Flir ui Image",
                gui_image_watch.1.borrow().clone(),
                Default::default(),
            ),
            settings_reciever: settings_watch.1,
            image_analysis_watch: Arc::new(image_analysis_watch.0),
            image_analysis_receiver: image_analysis_watch.1,
        };

        tokio::spawn(comm.clone().nal_intake_task());
        tokio::spawn(comm.clone().stream_ir_request_task());
        tokio::spawn(comm.clone().stream_visual_request_task());
        tokio::spawn(comm.clone().settings_update_task());
        tokio::spawn(comm.clone().analyze_image_task());

        debug!("Starting new flir communication system");

        comm
    }
    async fn nal_intake_task(self) {
        let mut nal_rx = self.net_tx.subscribe();
        let mut decoder = match Decoder::new() {
            Ok(d) => d,
            Err(_) => {
                error!("Nal intake task failed to make a decoder!");
                return;
            }
        };

        loop {
            let nal = match nal_rx.recv().await {
                Ok(NetMessage::FlirDriver(FlirDriverMessage::NalPacket(nal))) => {
                    let mut units = nal.clone();
                    to_bitstream_with_001_be::<u32>(&nal, &mut units);
                    units
                }
                _ => {
                    continue;
                }
            };

            for packet in nal_units(&nal) {
                if let Ok(Some(yuv)) = decoder.decode(&packet) {
                    let image_size = yuv.dimension_rgb();
                    let mut rgb_data = vec![0; image_size.0 * image_size.1 * 3];
                    yuv.write_rgb8(&mut rgb_data);
                    let image_data = match ImageBuffer::from_raw(
                        image_size.0 as u32,
                        image_size.1 as u32,
                        rgb_data,
                    ) {
                        Some(i) => i,
                        None => continue,
                    };

                    debug!("New image recieved from AFV");

                    let new_image = DynamicImage::ImageRgb8(image_data);
                    let _ = self.image_watch.send(new_image.clone());

                    let pixels = match new_image.as_rgb8() {
                        Some(i) => i,
                        None => continue,
                    }
                    .as_flat_samples();
                    let size = [image_size.0, image_size.1];
                    let color_image = ColorImage::from_rgb(size, pixels.as_slice());
                    let _ = self.gui_image_watch.send(color_image);
                }
            }
        }
    }
    async fn stream_ir_request_task(self) {
        loop {
            self.stream_ir_request.notified().await;
            sleep(Duration::from_secs(1)).await;
            let _ = self
                .net_tx
                .send(NetMessage::FlirDriver(FlirDriverMessage::OpenIrStream));
        }
    }
    async fn stream_visual_request_task(self) {
        loop {
            self.stream_visual_request.notified().await;
            sleep(Duration::from_secs(1)).await;
            let _ = self
                .net_tx
                .send(NetMessage::FlirDriver(FlirDriverMessage::OpenVisualStream));
        }
    }
    async fn settings_update_task(self) {
        let mut net_rx = self.net_tx.subscribe();

        loop{
            if let Ok(NetMessage::FlirOperator(FlirOperatorMessage::Settings(settings))) = net_rx.recv().await{
                let _ = self.settings_watch.send(settings);
            }
        }
    }
    async fn analyze_image_task(self){
        
    }
}

impl Renderable for FlirSystemCommunicator {
    fn render(&mut self, ui: &mut eframe::egui::Ui) {
        if !self.stream_ir && !self.stream_visual {
            if ui.button("Start streaming ir").clicked() {
                self.stream_ir = true;
            }
            if ui.button("Start streaming visual").clicked() {
                self.stream_visual = true;
            }
        }
        if self.stream_ir {
            self.stream_ir_request.notify_one();
            if ui.button("Stop streaming ir").clicked() {
                self.stream_ir = false;
            }
        }
        if self.stream_visual {
            self.stream_visual_request.notify_one();
            if ui.button("Stop streaming visual").clicked() {
                self.stream_visual = false;
            }
        }
        if ui.button("Poll settings").clicked(){
            self.adjustable_settings = self.settings_reciever.borrow_and_update().clone();
        }

        let drag = DragValue::new(&mut self.adjustable_settings.fliter_value).clamp_range(0..=255).speed(1.0);
        ui.add(drag);

        if ui.button("Send settings").clicked(){
            let _ = self.net_tx.send(NetMessage::FlirOperator(FlirOperatorMessage::SetSettings(self.adjustable_settings.clone())));
        }

        if let Ok(true) = self.gui_image_receiver.has_changed() {
            self.gui_texture = ui.ctx().load_texture(
                "Flir IR ui image",
                self.gui_image_receiver.borrow_and_update().to_owned(),
                Default::default(),
            );
        }

        ui.image(self.gui_texture.id(), ui.available_size());
    }
}
