use std::sync::Arc;

use common_std::gndgui::GuiElement;
use eframe::{epaint::TextureHandle, egui::{Ui, Window}};
use image::DynamicImage;
use image::io::Reader;
use tokio::{sync::RwLock, runtime::Runtime};

#[async_trait::async_trait]
pub trait DataSource{
    async fn get_encoded_image(&self) -> Vec<u8>;
    async fn get_image(&self) -> DynamicImage;
}
pub struct SampleImage{
    path: String,
}
impl SampleImage{
    pub fn new(path: String) -> SampleImage {
        Self{
            path,
        }
    }
}

#[async_trait::async_trait]
impl DataSource for SampleImage{
    async fn get_encoded_image(&self) -> Vec<u8> {
        todo!()
    }

    async fn get_image(&self) -> DynamicImage {
        Reader::open(self.path.clone()).expect("Could not open sample IR image").decode().expect("Could not decode sample IR image")
    }
}
pub struct Flir<D>{
    source: D,    
    rt: Arc<Runtime>,
    image_data: RwLock<DynamicImage>,
    gui_image: RwLock<Option<TextureHandle>>,
    is_open: RwLock<bool>,
}

impl<D:DataSource> Flir<D>{
    pub fn new(source: D, rt: Option<Arc<Runtime>>) -> Flir<D> {
        let rt = match rt{
            Some(r) => r,
            None => {
                Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not build tokio runtime"))
            },
        };
        Self{
            source,
            image_data: RwLock::new(DynamicImage::new_rgb8(1024, 920)),
            gui_image: RwLock::new(None),
            is_open: RwLock::new(false),
            rt,
        }
    }
    fn ui(&self, ui: &mut Ui){
        let texture = self.load_gui_image(ui);
        let size = ui.available_size();
        ui.image(texture.id(), size);
        
    }
    fn load_gui_image(&self, ui: &Ui) -> TextureHandle{
        let mut gui_image = self.gui_image.blocking_write();
        if let Some(i) =  &(*gui_image){
            return i.clone();
        }

        let image_data = self.image_data.blocking_read();
        let image = image_data.clone().into_rgba8();
        let pixels = image.as_flat_samples();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = eframe::egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        let texture = ui.ctx().load_texture("Flir output", color_image, Default::default());
        *gui_image = Some(texture.clone());
        texture
        
    }
    pub async fn update_image(&self){
        let image_data = self.source.get_image().await;
        let mut image_lock = self.image_data.write().await;
        let mut gui_lock = self.gui_image.write().await;
        *image_lock = image_data;
        *gui_lock = None;
    }
    pub fn update_image_blocking(&self){
        self.rt.block_on(self.update_image());
    }
}

impl<D:DataSource> GuiElement for Flir<D>{
    fn name(&self) -> String {
        String::from("FLIR Cam")
    }

    fn render(&self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
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