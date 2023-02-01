use std::{
    fmt,
    io::Error,
    net::ToSocketAddrs,
    sync::Arc, cmp::Ordering, collections::HashSet,
};

use async_trait::async_trait;
use common_std::gndgui::GuiElement;
use eframe::{
    egui::{Ui, Window},
    epaint::TextureHandle,
};
use futures::StreamExt;
use glam::Vec2;
use image::{io::Reader, DynamicImage, GenericImageView, ImageBuffer, GenericImage};
use openh264::{decoder::Decoder, nal_units, to_bitstream_with_001_be};
use retina::client::{self, PlayOptions, SessionOptions, SetupOptions};
use tokio::time::Duration;
use tokio::{runtime::Runtime, sync::RwLock, time::sleep};
use url::Url;

#[async_trait]
pub trait DataSource: Send + Sync {
    async fn get_image(&self) -> DynamicImage;
}
pub struct SampleImage {
    path: String,
}
impl SampleImage {
    pub fn new(path: String) -> SampleImage {
        Self { path }
    }
}
#[derive(Debug)]
pub enum AnnexConversionError {
    NalLenghtParseError,
    NalUnitExtendError,
    IoError(Error),
}
pub struct RtspStream {
    url: Url,
    oversample: u32,
}

impl RtspStream {
    pub fn new<T: ToSocketAddrs + fmt::Display>(addr: T, oversample: u32) -> RtspStream {
        let url = Url::parse(&format!("rtsp://:@{}:554/avc", addr)).expect("Faulty ip addr");
        Self { url, oversample }
    }
    pub fn new_url(url: &str, oversample: u32) -> RtspStream {
        Self {
            url: Url::parse(url).expect("Invalid url"),
            oversample,
        }
    }
}

#[async_trait]
impl DataSource for RtspStream {
    async fn get_image(&self) -> DynamicImage {
        // We must first attempt to stream an image from the flir
        let mut options = SessionOptions::default();
        options = options.user_agent(String::from("demo"));

        let mut session = client::Session::describe(self.url.clone(), options)
            .await
            .expect("Could not establish session with A50");
        let options = SetupOptions::default();
        session
            .setup(0, options)
            .await
            .expect("Could not initiate stream with A50");
        let options = PlayOptions::default();
        let err = format!("Could not start playing string {}", 0);
        let play = session.play(options).await.expect(&err);
        let demux = play.demuxed().expect("Could not demux the playing stream");
        tokio::pin!(demux);

        // Now we will pull frames until we can build a successful picture
        let mut encoded_frames: Vec<u8> = Vec::with_capacity(100000);
        let mut nal_packets = Vec::with_capacity(100000);
        let mut rgb_image = None;
        let mut decoder = Decoder::new().expect("Could not make decoder");
        let mut successes = 0;
        while let None = rgb_image {
            let frame = demux.next().await;
            let frame = match frame {
                Some(f) => f,
                None => continue,
            };

            match frame {
                Ok(f) => {
                    match f {
                        retina::codec::CodecItem::VideoFrame(v) => {
                            // println!("Successfully received frame");
                            encoded_frames.extend_from_slice(v.data());
                        }
                        retina::codec::CodecItem::AudioFrame(_) => {}
                        retina::codec::CodecItem::MessageFrame(_) => {}
                        retina::codec::CodecItem::Rtcp(_) => {}
                        _ => {}
                    }
                }
                Err(_) => {
                    println!("Get image error");
                    break;
                }
            }

            nal_packets.clear();

            to_bitstream_with_001_be::<u32>(&encoded_frames, &mut nal_packets);

            println!(
                "Attempting decode with {} nal packets",
                nal_units(&nal_packets).count()
            );

            for packet in nal_units(&nal_packets) {
                match decoder.decode(packet) {
                    Ok(y) => match y {
                        Some(y) => {
                            if successes < self.oversample {
                                successes += 1;
                            } else {
                                let image_size = y.dimension_rgb();
                                let mut rgb_data = vec![0; image_size.0 * image_size.1 * 3];
                                y.write_rgb8(&mut rgb_data);
                                rgb_image = Some(DynamicImage::ImageRgb8(
                                    ImageBuffer::from_raw(
                                        image_size.0 as u32,
                                        image_size.1 as u32,
                                        rgb_data,
                                    )
                                    .expect("Could not translate to image crate"),
                                ));
                            }
                            println!("Successful picture decode!");
                        }
                        None => {}
                    },
                    Err(_) => {}
                }
            }
        }

        match rgb_image {
            Some(i) => {
                return i;
            }
            None => {
                println!("Failed to get image");
                let image_size = (1, 1);
                let rgb_data = vec![255, 255, 255];
                return DynamicImage::ImageRgb8(
                    ImageBuffer::from_raw(image_size.0 as u32, image_size.1 as u32, rgb_data)
                        .expect("Could not translate to image crate"),
                );
            }
        }
    }
}

#[async_trait]
impl DataSource for SampleImage {
    async fn get_image(&self) -> DynamicImage {
        Reader::open(self.path.clone())
            .expect("Could not open sample IR image")
            .decode()
            .expect("Could not decode sample IR image")
    }
}
pub struct A50<D> {
    source: D,
    rt: Arc<Runtime>,
    image_data: RwLock<DynamicImage>,
    gui_image: RwLock<Option<TextureHandle>>,
    is_open: RwLock<bool>,
    refresh_toggle: RwLock<Option<Arc<bool>>>,
    best_dir_mean: RwLock<f32>,
    best_dir_median: RwLock<f32>,
}

impl<D: DataSource + 'static> A50<D> {
    pub fn new(source: D, rt: Option<Arc<Runtime>>) -> A50<D> {
        let rt = match rt {
            Some(r) => r,
            None => Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Could not build tokio runtime"),
            ),
        };
        Self {
            source,
            image_data: RwLock::new(DynamicImage::new_rgb8(1024, 920)),
            gui_image: RwLock::new(None),
            is_open: RwLock::new(false),
            rt,
            refresh_toggle: RwLock::new(None),
            best_dir_mean: RwLock::new(0.0),
            best_dir_median: RwLock::new(0.0),
        }
    }
    pub async fn update_best_dir_mean(a50: Arc<Self>) {
        let mut image = a50.image_data.write().await;
        let mut vectors = vec![];
        let mut pixels = Vec::with_capacity(image.as_bytes().len()/3);
        for pix in image.clone().pixels() {
            if pix.2[0] < 100 {
                let mut pix = pix.clone();
                pix.2[0] = 0;
                pix.2[1] = 0;
                pix.2[2] = 0;
                image.put_pixel(pix.0, pix.1, pix.2);
                continue;
            }
            pixels.push(pix);
        }
        if pixels.len() == 0{
            return;
        }

        pixels.sort_unstable_by(|a,b| {
            if a.1 > b.1{
                return Ordering::Less;
            }
            if a.1 == b.1{
                return Ordering::Equal;
            }
            Ordering::Greater
        });

        let mut top_index = pixels.len() - 1;
        let mut bottom_index = 0;
        while top_index > bottom_index{
            let top_pix = pixels[top_index];
            let bottom_pix = pixels[bottom_index];
            let top_pos = Vec2::new(top_pix.0 as f32, top_pix.1 as f32);
            let bottom_pos = Vec2::new(bottom_pix.0 as f32, bottom_pix.1 as f32);
            vectors.push(top_pos - bottom_pos);
            top_index -= 1;
            bottom_index += 1;
        }
        
        let total_vec: Vec2 = vectors.iter().sum();
        *a50.best_dir_mean.write().await = -(total_vec / vectors.len() as f32).x;
    }
    pub async fn update_best_dir_median(a50: Arc<Self>) {
        let mut image = a50.image_data.write().await;
        let mut vectors = vec![];
        let mut pixels = Vec::with_capacity(image.as_bytes().len()/3);
        for pix in image.clone().pixels() {
            if pix.2[0] < 100 {
                let mut pix = pix.clone();
                pix.2[0] = 0;
                pix.2[1] = 0;
                pix.2[2] = 0;
                image.put_pixel(pix.0, pix.1, pix.2);
                continue;
            }
            pixels.push(pix);
        }
        if pixels.len() == 0{
            return;
        }

        let mut y_vals = HashSet::new();
        for pix in pixels.iter(){
            y_vals.insert(pix.1);
        }

        let mut y_vals:Vec<(u32, u32)> = y_vals.iter().map(|v| (0, *v)).collect();
        y_vals.sort_unstable();
        y_vals.reverse();
        for (median, y_val) in y_vals.iter_mut(){
            let mut x_vals:Vec<u32> = pixels.iter().filter(|pix| pix.1 == *y_val).map(|pix| pix.0).collect();
            x_vals.sort_unstable();
            *median = x_vals[x_vals.len()/2];
        }
        
        let mut top_index = y_vals.len() - 1;
        let mut bottom_index = 0;
        while top_index > bottom_index{
            let top_pos = y_vals[top_index];
            let bottom_pos = y_vals[bottom_index];
            let top_pos = Vec2::new(top_pos.0 as f32, top_pos.1 as f32);
            let bottom_pos = Vec2::new(bottom_pos.0 as f32, bottom_pos.1 as f32);
            // Remember, images are indexed from the top-left to bottom-right
            let vec = top_pos - bottom_pos;
            vectors.push(vec);
            top_index -= 1;
            bottom_index += 1;
        }
        
        let total_vec: Vec2 = vectors.iter().sum();
        *a50.best_dir_median.write().await = -(total_vec / vectors.len() as f32).x;
    }
    async fn is_open_async(&self) -> bool {
        *self.is_open.read().await
    }
    fn ui(&self, ui: &mut Ui) {
        let texture = self.load_gui_image(ui);
        let size = ui.available_size()*0.75;
        ui.image(texture.id(), size);
        ui.label(format!("Mean recommended dir: {}", self.best_dir_mean.blocking_read()));
        ui.label(format!("Median recommended dir: {}", self.best_dir_median.blocking_read()));
    }
    fn load_gui_image(&self, ui: &Ui) -> TextureHandle {
        let mut gui_image = self.gui_image.blocking_write();
        if let Some(i) = &(*gui_image) {
            return i.clone();
        }

        let image_data = self.image_data.blocking_read();
        let image = image_data.clone().into_rgba8();
        let pixels = image.as_flat_samples();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = eframe::egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        let texture = ui
            .ctx()
            .load_texture("Flir output", color_image, Default::default());
        *gui_image = Some(texture.clone());
        texture
    }
    pub async fn update_image(&self) {
        let image_data = self.source.get_image().await;
        let mut image_lock = self.image_data.write().await;
        let mut gui_lock = self.gui_image.write().await;
        *image_lock = image_data;
        *gui_lock = None;
    }
    pub fn update_image_blocking(&self) {
        self.rt.block_on(self.update_image());
    }
    pub fn refresh_interval(a50: Arc<Self>, interval: Duration) {
        let toggle = Arc::new(false);
        a50.rt
            .spawn(A50::refresh_task(a50.clone(), interval, toggle.clone()));
        let mut toggle_lock = a50.refresh_toggle.blocking_write();
        *toggle_lock = Some(toggle);
    }
    pub async fn refresh_task(a50: Arc<Self>, inerval: Duration, toggle: Arc<bool>) {
        while Arc::strong_count(&toggle) > 1 {
            sleep(inerval).await;
            if a50.is_open_async().await{
                a50.update_image().await;
                A50::update_best_dir_mean(a50.clone()).await;
                A50::update_best_dir_median(a50.clone()).await;
            }
        }
    }
}

impl<D: DataSource + 'static> GuiElement for A50<D> {
    fn name(&self) -> String {
        String::from("FLIR Cam")
    }

    fn render(&self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let mut open = true;
        Window::new(self.name())
            .open(&mut open)
            .constrain(true)
            .hscroll(true)
            .vscroll(true)
            .resizable(true)
            .show(ctx, |ui| self.ui(ui));
        self.set_open(open);
    }

    fn is_open(&self) -> bool {
        *self.is_open.blocking_read()
    }

    fn set_open(&self, status: bool) {
        *self.is_open.blocking_write() = status
    }
}
