use std::sync::Arc;

use eframe::{
    egui::{
        self,
        plot::{Arrows, PlotImage, PlotPoint, Points},
        DragValue, TextureHandle, Ui,
    },
    epaint::{Color32, ColorImage},
};
use image::DynamicImage;
use log::{debug, error};
use openh264::decoder::Decoder;

use tokio::{
    sync::{broadcast, watch, Notify},
    time::{sleep, Duration},
};

use crate::{
    drivers::flir::FlirDriverMessage,
    network::NetMessage,
    operators::flir::{FlirAnalysis, FlirOperator, FlirOperatorMessage, FlirOperatorSettings},
    ui::Renderable,
};

#[derive(Clone)]
pub struct FlirSystemCommunicator {
    net_tx: broadcast::Sender<NetMessage>,

    settings_watch: Arc<watch::Sender<FlirOperatorSettings>>,
    settings_reciever: watch::Receiver<FlirOperatorSettings>,

    image_watch: Arc<watch::Sender<DynamicImage>>,

    image_analysis_watch: Arc<watch::Sender<FlirAnalysis>>,
    image_analysis_receiver: watch::Receiver<FlirAnalysis>,

    gui_image_watch: Arc<watch::Sender<ColorImage>>,
    gui_image_receiver: watch::Receiver<ColorImage>,

    stream_ir_request: Arc<Notify>,
    stream_visual_request: Arc<Notify>,
    auto_target_request_notify: Arc<Notify>,

    //Ui parameters
    adjustable_settings: FlirOperatorSettings,
    stream_ir: bool,
    stream_visual: bool,
    auto_target: bool,
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
            auto_target_request_notify: Default::default(),
            auto_target: Default::default(),
        };

        tokio::spawn(comm.clone().nal_intake_task());
        tokio::spawn(comm.clone().stream_ir_request_task());
        tokio::spawn(comm.clone().stream_visual_request_task());
        tokio::spawn(comm.clone().settings_update_task());
        tokio::spawn(comm.clone().auto_target_request_task());
        tokio::spawn(comm.clone().analysis_intake_task());

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
                Ok(NetMessage::FlirDriver(FlirDriverMessage::NalPacket(nal))) => nal,
                _ => {
                    continue;
                }
            };

            let image = match FlirOperator::process_nal_data(nal, &mut decoder) {
                Some(i) => i,
                None => continue,
            };

            let _ = self.image_watch.send(image.clone());

            let pixels = match image.as_rgb8() {
                Some(i) => i,
                None => continue,
            }
            .as_flat_samples();
            let size = [image.width() as usize, image.height() as usize];
            let color_image = ColorImage::from_rgb(size, pixels.as_slice());
            let _ = self.gui_image_watch.send(color_image);
        }
    }
    async fn analysis_intake_task(self) {
        let mut net_rx = self.net_tx.subscribe();

        loop {
            if let Ok(NetMessage::FlirOperator(FlirOperatorMessage::Analysis(analysis))) =
                net_rx.recv().await
            {
                let _ = self.image_analysis_watch.send(analysis);
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
    async fn auto_target_request_task(self) {
        loop {
            self.auto_target_request_notify.notified().await;
            sleep(Duration::from_secs(1)).await;
            let _ = self
                .net_tx
                .send(NetMessage::FlirOperator(FlirOperatorMessage::AutoTarget));
        }
    }
    async fn settings_update_task(self) {
        let mut net_rx = self.net_tx.subscribe();

        loop {
            if let Ok(NetMessage::FlirOperator(FlirOperatorMessage::Settings(settings))) =
                net_rx.recv().await
            {
                let _ = self.settings_watch.send(settings);
            }
        }
    }
    fn plot_image(&mut self, ui: &mut Ui) {
        if let Ok(true) = self.gui_image_receiver.has_changed() {
            self.gui_texture = ui.ctx().load_texture(
                "Flir IR ui image",
                self.gui_image_receiver.borrow_and_update().to_owned(),
                Default::default(),
            );
        }
        let texture = self.gui_texture.clone();
        let gui_image_size = texture.size_vec2();
        let analysis = self.image_analysis_receiver.borrow_and_update().clone();

        let lower_centroid = [
            analysis.lower_centroid[0] as f64,
            -analysis.lower_centroid[1] as f64,
        ];
        let upper_centroid = [
            analysis.upper_centroid[0] as f64,
            -analysis.upper_centroid[1] as f64,
        ];
        let points = Points::new(vec![lower_centroid, upper_centroid])
            .shape(egui::plot::MarkerShape::Circle)
            .filled(true)
            .radius(5.0)
            .name("Centroids");
        let centroidal_arrow = Arrows::new(vec![upper_centroid], vec![lower_centroid])
            .color(Color32::RED)
            .name("Fire Axis");

        let delta_angles = analysis.angle_change;
        let center = [
            gui_image_size.x as f64 / 2.0,
            -gui_image_size.y as f64 / 2.0,
        ];
        let rot_x = [
            (gui_image_size.x + 5.0 * delta_angles[0]) as f64 / 2.0,
            -gui_image_size.y as f64 / 2.0,
        ];
        let rot_y = [
            gui_image_size.x as f64 / 2.0,
            (-gui_image_size.y + 5.0 * delta_angles[1]) as f64 / 2.0,
        ];
        let rotation_arrow = Arrows::new(vec![center, center], vec![rot_x, rot_y])
            .color(Color32::GOLD)
            .name("Rotation Arrow");

        egui::widgets::plot::Plot::new("Flir plot")
            .show_background(false)
            .data_aspect(1.0)
            .include_x(gui_image_size.x)
            .include_y(-gui_image_size.y)
            .label_formatter(|_name, value| format!("    {:.0}, {:.0}", value.x, value.y.abs()))
            .y_axis_formatter(|y, _range| y.abs().to_string())
            .show(ui, |ui| {
                let image = PlotImage::new(
                    texture.id(),
                    PlotPoint::new(gui_image_size.x / 2.0, -gui_image_size.y / 2.0),
                    gui_image_size,
                );
                ui.image(image);
                ui.points(points);
                ui.arrows(centroidal_arrow);
                ui.arrows(rotation_arrow);
            });
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
        if ui.button("Poll settings").clicked() {
            self.adjustable_settings = self.settings_reciever.borrow_and_update().clone();
        }
        if self.auto_target {
            self.auto_target_request_notify.notify_one();
            if ui.button("Stop auto target").clicked() {
                self.auto_target = false;
            }
        } else {
            if ui.button("Start auto target").clicked() {
                self.auto_target = true;
            }
        }

        let drag = DragValue::new(&mut self.adjustable_settings.fliter_value)
            .clamp_range(0..=255)
            .speed(1.0);
        ui.add(drag);

        if ui.button("Send settings").clicked() {
            let _ = self
                .net_tx
                .send(NetMessage::FlirOperator(FlirOperatorMessage::SetSettings(
                    self.adjustable_settings.clone(),
                )));
        }

        self.plot_image(ui)
    }
}
