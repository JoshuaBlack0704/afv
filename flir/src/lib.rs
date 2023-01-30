use std::{sync::Arc, io::{Cursor, Read, Error}, net::ToSocketAddrs, fmt, slice::from_raw_parts};

use async_trait::async_trait;
use common_std::gndgui::GuiElement;
use eframe::{epaint::TextureHandle, egui::{Ui, Window}};
use futures::StreamExt;
use image::{DynamicImage, ImageBuffer, RgbImage};
use image::io::Reader;
use openh264::{decoder::Decoder, nal_units, to_bitstream_with_001_be};
use retina::client::{SessionOptions, self, SetupOptions, PlayOptions};
use tokio::{sync::RwLock, runtime::Runtime};
use url::Url;

#[async_trait]
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
#[derive(Debug)]
pub enum AnnexConversionError{
    NalLenghtParseError,
    NalUnitExtendError,
    IoError(Error),
}
pub struct RtspStream{
    url: Url,
}

impl RtspStream{
    pub fn new<T:ToSocketAddrs + fmt::Display>(addr: T) -> RtspStream {
        let url = Url::parse(&format!("rtsp://:@{}:554/avc", addr)).expect("Faulty ip addr");
        Self{
            url,
        }
    }
    pub fn new_url(url: &str) -> RtspStream {
        Self{
            url: Url::parse(url).expect("Invalid url"),
        }
    }
    /// Converts from AVCC format to annex b format
    pub fn avcc_to_annex_b_cursor(
        data: &[u8],
        nal_units: &mut Vec<u8>,
    ) -> Result<(), AnnexConversionError> {
        let mut data_cursor = Cursor::new(data);
        let mut nal_lenght_bytes = [0u8; 4];
        while let Ok(bytes_read) = data_cursor.read(&mut nal_lenght_bytes) {
            if bytes_read == 0 {
                break;
            }
            if bytes_read != nal_lenght_bytes.len() || bytes_read == 0 {
                return Err(AnnexConversionError::NalLenghtParseError);
            }
            let nal_length = u32::from_be_bytes(nal_lenght_bytes) as usize;
            nal_units.push(0);
            nal_units.push(0);
            nal_units.push(1);

            if nal_length == 0 {
                return Err(AnnexConversionError::NalLenghtParseError);
            }
            let mut nal_unit = vec![0u8; nal_length];
            let bytes_read = data_cursor.read(&mut nal_unit);
            match bytes_read {
                Ok(bytes_read) => {
                    nal_units.extend_from_slice(&nal_unit[0..bytes_read]);
                    //TODO: this is never called so we don't ever detect EOF
                    if bytes_read == 0 {
                        break;
                    } else if bytes_read < nal_unit.len() {
                        return Err(AnnexConversionError::NalUnitExtendError);
                    }
                }
                Err(e) => return Err(AnnexConversionError::IoError(e)),
            };
        }
        Ok(())
    }
}

#[async_trait]
impl DataSource for RtspStream{
    async fn get_encoded_image(&self) -> Vec<u8> {
        
        let mut options = SessionOptions::default();
        options = options.user_agent(String::from("demo"));
        options = options.creds(Some(client::Credentials { username: String::from("demo"), password: String::from("demo") }));

        let mut session = client::Session::describe(self.url.clone(), options).await.expect("Could not establish session with A50");
        let mut options = SetupOptions::default();
        options = options.transport(client::Transport::Udp(Default::default()));
        session.setup(0, options).await.expect("Could not initiate stream with A50");
        let options = PlayOptions::default();
        let err = format!("Could not start playing string {}", 0);
        let play = session.play(options).await.expect(&err);
        // tokio::pin!(play);
        // while let Some(y) = play.next().await{
        //     let y:Vec<retina::rtcp::PacketRef> = match y.unwrap(){
        //         client::PacketItem::Rtp(r) => continue,
        //         client::PacketItem::Rtcp(r) => continue,
        //         _ => todo!(),
        //     };
        //     for pkt in y.iter(){
        //         let sender = pkt.as_sender_report().unwrap().unwrap();
        //         println!("{:?}", sender.count());
        //     }
        // }
        let demux = play.demuxed().expect("Could not demux the playing stream");
        tokio::pin!(demux);
        let mut encoded_frames:Vec<u8> = Vec::with_capacity(100000);
        for _ in 0..50{
            if let Some(item) = demux.next().await{
                match item{
                    Ok(e) => {
                        match e{
                            retina::codec::CodecItem::VideoFrame(v) => {
                                println!("Got frame from flir");
                                encoded_frames.extend_from_slice(v.data());
                            },
                            retina::codec::CodecItem::AudioFrame(_) => todo!(),
                            retina::codec::CodecItem::MessageFrame(_) => todo!(),
                            retina::codec::CodecItem::Rtcp(_) => continue,
                            _ => todo!(),
                        }
                    },
                    Err(_) => todo!(),
                }
            }
        }

        encoded_frames
    }

    async fn get_image(&self) -> DynamicImage {
        let encoded_image = self.get_encoded_image().await;
        let mut nal:Vec<u8> = Vec::with_capacity(encoded_image.len());
        // RtspStream::avcc_to_annex_b_cursor(&encouded_image, &mut nal).expect("Could not translate to annex b encoding");
        to_bitstream_with_001_be::<u32>(&encoded_image, &mut nal);
        
        let mut decoder = Decoder::new().expect("Could not make decoder");
        let mut size = (1,1);
        let mut rgb = vec![255,255,255];
        for packet in nal_units(&nal){
            let res = decoder.decode(packet);
            match res{
                Ok(y) => {
                    match y{
                        Some(yuv) => {
                            size = yuv.dimension_rgb();
                            rgb = vec![0;size.0*size.1*3];
                            yuv.write_rgb8(&mut rgb);
                            println!("Successful picture!");
                        },
                        None => println!("Not enough info for picture"),
                    }
                },
                Err(e) => println!("{}", e),
            }
        }

        let image:RgbImage = ImageBuffer::from_raw(size.0 as u32, size.1 as u32, rgb).expect("Could not translate to image crate");

        DynamicImage::ImageRgb8(image)
        
    }
}

#[async_trait]
impl DataSource for SampleImage{
    async fn get_encoded_image(&self) -> Vec<u8> {
        todo!()
    }

    async fn get_image(&self) -> DynamicImage {
        Reader::open(self.path.clone()).expect("Could not open sample IR image").decode().expect("Could not decode sample IR image")
    }
}
pub struct A50<D>{
    source: D,    
    rt: Arc<Runtime>,
    image_data: RwLock<DynamicImage>,
    gui_image: RwLock<Option<TextureHandle>>,
    is_open: RwLock<bool>,
}

impl<D:DataSource> A50<D>{
    pub fn new(source: D, rt: Option<Arc<Runtime>>) -> A50<D> {
        let rt = match rt{
            Some(r) => r,
            None => {
                Arc::new(tokio::runtime::Builder::new_current_thread().enable_all().build().expect("Could not build tokio runtime"))
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

impl<D:DataSource> GuiElement for A50<D>{
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