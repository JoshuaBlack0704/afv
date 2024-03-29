use std::sync::Arc;

use afv_internal::FLIR_TURRET_PORT;
use glam::Vec2;
use image::{DynamicImage, ImageBuffer};
use log::{error, info, trace};
use openh264::{decoder::Decoder, nal_units, to_bitstream_with_001_be};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{broadcast, watch},
    time::{sleep, Duration, Instant},
};

use crate::{
    drivers::{flir::FlirDriver, turret::TurretDriverMessage},
    network::NetMessage,
};

pub const BROADCAST_SETTINGS_INTERVAL: Duration = Duration::from_secs(5);
pub const AUTO_TARGET_REQUEST_INTERVAL: Duration = Duration::from_secs(1);
pub const AUTO_TARGET_REQUEST_WAIT: Duration = Duration::from_millis(500);
pub const FLIRFOV: (f32, f32) = (29.0, 22.0);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FlirOperatorMessage {
    Settings(FlirOperatorSettings),
    SetSettings(FlirOperatorSettings),
    Analysis(Option<FlirAnalysis>),
    AutoTarget,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FlirOperatorSettings {
    pub fliter_value: u8,
    pub interations: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct FlirAnalysis {
    pub lower_centroid: [f32; 2],
    pub upper_centroid: [f32; 2],
    pub angle_change: [f32; 2],
}

impl Default for FlirOperatorSettings {
    fn default() -> Self {
        Self {
            fliter_value: 200,
            interations: 2,
        }
    }
}

#[derive(Clone)]
/// The FlirOperator is the main control system that reads the FLIR A50's image data and commands the FLIR turret
/// to track fire signatures.
pub struct FlirOperator {
    net_tx: broadcast::Sender<NetMessage>,
    settings_watch: Arc<watch::Sender<FlirOperatorSettings>>,
    image_watch: Arc<watch::Sender<DynamicImage>>,
    auto_target_watch: Arc<watch::Sender<Instant>>,
    flir_driver: FlirDriver,
}

impl FlirOperator {
    /// Creates and adds a new FlirOperator to the main bus.
    pub async fn new(net_tx: broadcast::Sender<NetMessage>) -> FlirOperator {
        let operator = Self {
            flir_driver: FlirDriver::new(net_tx.clone(), true).await,
            net_tx,
            settings_watch: Arc::new(watch::channel(Default::default()).0),
            image_watch: Arc::new(watch::channel(Default::default()).0),
            auto_target_watch: Arc::new(watch::channel(Instant::now()).0),
        };

        tokio::spawn(operator.clone().settings_update_task());
        tokio::spawn(operator.clone().settings_broadcast_task());
        tokio::spawn(operator.clone().nal_intake_task());
        tokio::spawn(operator.clone().analyze_image_task());
        tokio::spawn(operator.clone().auto_target_task());
        tokio::spawn(operator.clone().auto_target_watch_task());
        operator
    }
    /// Updates the time of last request for the auto target system
    async fn auto_target_watch_task(self) {
        let mut net_rx = self.net_tx.subscribe();

        loop {
            if let Ok(NetMessage::FlirOperator(FlirOperatorMessage::AutoTarget)) =
                net_rx.recv().await
            {
                let _ = self.auto_target_watch.send(Instant::now());
            }
        }
    }
    /// Updates the FlirOperators's internal settings when the correct messages is received on the main bus
    async fn settings_update_task(self) {
        let mut net_rx = self.net_tx.subscribe();
        loop {
            if let Ok(NetMessage::FlirOperator(FlirOperatorMessage::SetSettings(settings))) =
                net_rx.recv().await
            {
                let _ = self.settings_watch.send(settings);
            }
        }
    }
    /// The main auto targeting task that, when enabled, commands the FLIR turret
    async fn auto_target_task(self) {
        let mut net_rx = self.net_tx.subscribe();
        let mut auto_target_watch = self.auto_target_watch.subscribe();
        sleep(AUTO_TARGET_REQUEST_INTERVAL + Duration::from_secs(1)).await;

        let mut loop_count: f32 = 0.0;

        loop {
            if Instant::now().duration_since(*auto_target_watch.borrow_and_update())
                > AUTO_TARGET_REQUEST_INTERVAL
            {
                let _ = auto_target_watch.changed().await;
                info!("Staring flir operator auto target");
                continue;
            }

            match net_rx.recv().await {
                Ok(NetMessage::FlirOperator(FlirOperatorMessage::Analysis(Some(analysis)))) => {
                    info!(
                        "Commanding flir turret to changle {:?} deg",
                        analysis.angle_change
                    );
                    let _ = self.net_tx.send(NetMessage::TurretDriver(
                        TurretDriverMessage::SetAngleChange(
                            FLIR_TURRET_PORT,
                            analysis.angle_change,
                        ),
                    ));
                    // let _ = self.net_tx.send(NetMessage::TurretDriver(TurretDriverMessage::SetAngle(NOZZLE_TURRET_PORT, analysis.angle_change)));
                }
                Ok(NetMessage::FlirOperator(FlirOperatorMessage::Analysis(None))) => {
                    let dir = loop_count.sin().signum();
                    let angle: f32 = 30.0 * dir;
                    info!("Commanding flir turret rotate {} deg", angle);
                    let _ = self.net_tx.send(NetMessage::TurretDriver(
                        TurretDriverMessage::SetAngleChange(FLIR_TURRET_PORT, [angle, 0.0]),
                    ));
                    // let _ = self.net_tx.send(NetMessage::TurretDriver(TurretDriverMessage::SetAngle(NOZZLE_TURRET_PORT, [angle, 0.0])));
                }
                _ => {}
            }

            loop_count += 0.005;
        }
    }
    /// When polled with the correct message on the main bus, sends the FlirOperators internal settings
    /// on the main bus
    async fn settings_broadcast_task(self) {
        let mut settings_rx = self.settings_watch.subscribe();
        loop {
            let _ = self
                .net_tx
                .send(NetMessage::FlirOperator(FlirOperatorMessage::Settings(
                    settings_rx.borrow_and_update().clone(),
                )));
            sleep(BROADCAST_SETTINGS_INTERVAL).await;
        }
    }
    /// When auto targeting is enables, receives compeleted IR images and performs the fire detection algorithm
    async fn analyze_image_task(self) {
        let mut image_rx = self.image_watch.subscribe();
        let mut settings_rx = self.settings_watch.subscribe();

        loop {
            sleep(AUTO_TARGET_REQUEST_INTERVAL).await;
            // If a new IR image is available
            match image_rx.changed().await {
                Ok(_) => {}
                Err(_) => continue,
            };

            // We clone here to avoid holding onto a mutexed lock. VERY IMPORTANT
            let image = image_rx.borrow_and_update().clone().into_rgb8();
            let settings = settings_rx.borrow_and_update().clone();

            // Create a storage image to hold our filtering in
            let mut filtered_image =
                DynamicImage::new_rgb8(image.width(), image.height()).into_rgb8();
            let mut pixels = Vec::with_capacity((image.width() * image.height() * 3) as usize);

            for (x, y, pix) in image
                .enumerate_pixels()
                .filter(|(_, _, pix)| pix.0[0] > settings.fliter_value)
            {
                filtered_image.put_pixel(x, y, *pix);
                pixels.push((x, y))
            }

            // If we have insufficent signature, we exit and wait for a new image
            if pixels.len() <= 10 {
                let _ = self
                    .net_tx
                    .send(NetMessage::FlirOperator(FlirOperatorMessage::Analysis(
                        None,
                    )));
                continue;
            }

            // The main analysis is a two dimensional median centroid approach
            // Essentially we split the signature pixels into a top and bottom, left and right.
            // Doing this means we can pick out two centroids that roughly indicate the top and bottom
            // of the fire

            // NOTE: The images from the image create are indexed starting at the top right meaning
            // that the higher part of the fire is actually the lower centroid

            pixels.sort_unstable_by_key(|(_, y)| *y);
            let pix_count = pixels.len();

            let (lpix, upix) = pixels.split_at_mut(pix_count / 2);

            let lower_y_median = lpix[lpix.len() / 2];
            lpix.sort_unstable_by_key(|(x, _)| *x);
            let lower_x_median = lpix[lpix.len() / 2];
            let lower_centroid = Vec2::new(lower_x_median.0 as f32, lower_y_median.1 as f32);

            let mut upix = upix.to_vec();
            let mut upper_centroid = Default::default();
            for _ in 0..settings.interations {
                let upper_y_median = match upix.get(upix.len() / 2) {
                    Some(p) => *p,
                    None => (0, 0),
                };
                upix.sort_unstable_by_key(|(x, _)| *x);
                let upper_x_median = match upix.get(upix.len() / 2) {
                    Some(p) => *p,
                    None => (0, 0),
                };
                upper_centroid = Vec2::new(upper_x_median.0 as f32, upper_y_median.1 as f32);

                upix.sort_unstable_by_key(|(_, y)| *y);
                let (_, upper) = upix.split_at(upix.len() / 2);
                upix = upper.to_vec();
            }

            // Once we have our upper centroid (bottom of fire) we can use the FOV of the FLIR A50
            // to estimate an angular displacement of the Flir turret to aim us at the fire
            let target_centroid = upper_centroid;
            let image_width = image.width();
            let image_height = image.height();
            let center = Vec2::new((image_width as f32) / 2.0, (image_height as f32) / 2.0);
            let delta_pix = (target_centroid - center) * Vec2::new(1.0, 1.0);
            let deg_pix_x = (FLIRFOV.0 * 2.0) / image_width as f32;
            let deg_pix_y = (FLIRFOV.1 * 2.0) / image_height as f32;

            let delta_x = delta_pix.x * deg_pix_x;
            let delta_y = delta_pix.y * deg_pix_y;

            let analysis = FlirAnalysis {
                lower_centroid: [lower_centroid.x, lower_centroid.y],
                upper_centroid: [upper_centroid.x, upper_centroid.y],
                angle_change: [delta_x, -delta_y],
            };

            let _ = self
                .net_tx
                .send(NetMessage::FlirOperator(FlirOperatorMessage::Analysis(
                    Some(analysis),
                )));
        }
    }
    /// When auto targeting is enabled, starts constructing images from the [crate::drivers::flir::FlirDriver] ir nal stream
    async fn nal_intake_task(self) {
        let mut nal_rx = self.flir_driver.ir_stream_rx();
        let mut auto_target_watch = self.auto_target_watch.subscribe();
        let mut decoder = match Decoder::new() {
            Ok(d) => d,
            Err(_) => {
                error!("Nal intake task failed to make a decoder!");
                return;
            }
        };

        sleep(AUTO_TARGET_REQUEST_INTERVAL + Duration::from_secs(1)).await;

        loop {
            if Instant::now().duration_since(*auto_target_watch.borrow_and_update())
                > AUTO_TARGET_REQUEST_INTERVAL
            {
                info!("Stopping flir operator image intake");
                let _ = auto_target_watch.changed().await;
                info!("Staring flir operator image intake");
                nal_rx = self.flir_driver.ir_stream_rx();
                continue;
            }

            let nal = match nal_rx.recv().await {
                Ok(nal) => nal,
                _ => {
                    continue;
                }
            };

            let image = match FlirOperator::process_nal_data(nal, &mut decoder) {
                Some(i) => i,
                None => continue,
            };

            let _ = self.image_watch.send(image.clone());
        }
    }
    /// Takes nal data BEFORE breaking it with to_bitstream
    pub fn process_nal_data(nal_data: Vec<u8>, decoder: &mut Decoder) -> Option<DynamicImage> {
        let mut most_recent_image = None;

        let mut nal_bitstream = nal_data.clone();

        to_bitstream_with_001_be::<u32>(&nal_data, &mut nal_bitstream);

        for packet in nal_units(&nal_bitstream) {
            if let Ok(Some(yuv)) = decoder.decode(&packet) {
                let image_size = yuv.dimension_rgb();
                let mut rgb_data = vec![0; image_size.0 * image_size.1 * 3];
                yuv.write_rgb8(&mut rgb_data);
                let image_data =
                    match ImageBuffer::from_raw(image_size.0 as u32, image_size.1 as u32, rgb_data)
                    {
                        Some(i) => i,
                        None => continue,
                    };

                trace!("New image recieved from FLIR driver");

                most_recent_image = Some(DynamicImage::ImageRgb8(image_data));
            }
        }
        most_recent_image
    }
}
