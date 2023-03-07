use std::sync::Arc;

use eframe::{egui::{Ui, self, TopBottomPanel, CentralPanel, plot::{Points, Arrows, PlotImage, PlotPoint}}, epaint::{TextureHandle, Color32}};
use glam::Vec2;
use image::{ImageBuffer, DynamicImage};
use openh264::{to_bitstream_with_001_be, nal_units, decoder::Decoder};
use tokio::time::sleep;

use crate::messages::{AfvCtlMessage, NetworkMessages};

use super::AfvController;

impl AfvController{
    pub (super) async fn flir_stream_manager(self: Arc<Self>){
        loop{
            if let Some(_) = & (*self.flir_decoder.read().await){
                self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Network(NetworkMessages::FlirStream(*self.afv_uuid.read().await))).await;
            }
            sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
    fn get_gui_image(&self, ui: &mut Ui) -> TextureHandle{
        let mut gui_image = self.flir_gui_image.blocking_write();
        if let Some(i) = &(*gui_image){
            return i.clone();
        }

        let image;
        // Select processed or direct image
        match & (*self.flir_filtered_image.blocking_read()){
            Some(i) => {
               image = i.clone().into_rgb8(); 
            },
            None => {
                image = self.flir_image.blocking_read().clone().into_rgb8();
            },
        }
        let pixels = image.as_flat_samples();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = egui::ColorImage::from_rgb(size, pixels.as_slice());
        let handle = ui.ctx().load_texture("Flir Output", color_image, Default::default());

        *gui_image = Some(handle.clone());
        handle
    }
    pub (super) fn render_flir_display(&self, ui: &mut Ui){
        TopBottomPanel::top("Flir controls").show_inside(ui, |ui|{
            ui.horizontal(|ui|{
                // Stream controls
                let mut decoder = self.flir_decoder.blocking_write();
                match &mut (*decoder){
                    Some(_) => {
                        if ui.button("Stop Flir Stream").clicked(){
                            *decoder = None;
                        }
                    },
                    None => {
                        if ui.button("Start Flir Stream").clicked(){
                            if let Ok(d) = Decoder::new(){
                                *decoder = Some(d);
                            }
                        }
                    },
                }

                // Fliter controls
                let mut filtered_image = self.flir_filtered_image.blocking_write();
                match &(*filtered_image){
                    Some(_) => {
                        if ui.button("Hide filtered image").clicked(){
                            *filtered_image = None;
                        }
                    },
                    None => {
                        if ui.button("Show filtered image").clicked(){
                            *filtered_image = Some(DynamicImage::new_rgb8(100,100));
                        }
                    },
                }
                let mut filter_toggle = self.flir_filter.blocking_write();
                if !(*filter_toggle){
                    if ui.button("Enable filtering").clicked(){
                        *filter_toggle = true;
                    }
                }
                else {
                    if ui.button("Disable filtering").clicked(){
                        *filter_toggle = false;
                    }
                }
                let mut filter_level = self.flir_filter_level.blocking_write();
                let drag = egui::widgets::DragValue::new(&mut (*filter_level))
                    .clamp_range(0..=u8::MAX)
                    .prefix("Filter Level: ")
                    .speed(1.0);
                ui.add(drag);
                let mut filter_target_iterations = self.flir_target_iterations.blocking_write();
                let drag = egui::widgets::DragValue::new(&mut (*filter_target_iterations))
                    .clamp_range(0..=u8::MAX)
                    .prefix("Filter Target Iterations: ")
                    .speed(1.0);
                ui.add(drag);
                if ui.button("Send filter level").clicked(){
                    self.handle.spawn(self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Network(NetworkMessages::FlirFilterLevel(*filter_level))));
                    self.handle.spawn(self.bus.clone().send(self.bus_uuid, AfvCtlMessage::Network(NetworkMessages::FlirTargetIterations(*filter_target_iterations))));
                }
            });
        });
        CentralPanel::default().show_inside(ui, |ui|{
            self.plot_image(ui);
        });
    }
    pub (super) fn plot_image(&self, ui: &mut Ui){
        let texture = self.get_gui_image(ui);
        let gui_image_size = texture.size_vec2();
        let (lower_centroid, upper_centroid) = *self.flir_centroids.blocking_read();
        
        let lower_centroid = [lower_centroid.x as f64, -lower_centroid.y as f64];
        let upper_centroid = [upper_centroid.x as f64, -upper_centroid.y as f64];
        let points = Points::new(vec![lower_centroid, upper_centroid])
        .shape(egui::plot::MarkerShape::Circle)
        .filled(true)
        .radius(5.0)
        .name("Centroids");
        let arrow = Arrows::new(vec![upper_centroid], vec![lower_centroid])
        .color(Color32::RED)
        .name("Fire Axis");
        
        egui::widgets::plot::Plot::new("Flir plot")
            .show_background(false)
            .data_aspect(1.0)
            .include_x(gui_image_size.x)
            .include_y(-gui_image_size.y)
            .label_formatter(|_name, value| {format!("    {:.0}, {:.0}", value.x, value.y.abs())})
            .y_axis_formatter(|y, _range| {y.abs().to_string()})
            .show(ui, |ui|{
            let image = PlotImage::new(texture.id(), PlotPoint::new(gui_image_size.x/2.0,-gui_image_size.y/2.0), gui_image_size);
            ui.image(image);
            ui.points(points);
            ui.arrows(arrow);
        });
        
    }
    pub (super) async fn process_nal_packet(self: Arc<Self>, packet: Vec<u8>){
        if let Some(d) = &mut (*self.flir_decoder.write().await){
            let mut nal = Vec::with_capacity(packet.len());
            to_bitstream_with_001_be::<u32>(&packet, &mut nal);
            for packet in nal_units(&nal){
                if let Ok(Some(yuv)) = d.decode(&packet){
                    let image_size = yuv.dimension_rgb();
                    let mut rgb_data = vec![0; image_size.0*image_size.1*3];
                    yuv.write_rgb8(&mut rgb_data);
                    let image_data = match ImageBuffer::from_raw(
                        image_size.0 as u32,
                        image_size.1 as u32,
                        rgb_data,
                    ) {
                        Some(i) => i,
                        None => continue,
                    };
                    {
                        *self.flir_image.write().await = DynamicImage::ImageRgb8(image_data.clone());
                        *self.flir_gui_image.write().await = None;
                    }
                    // Since each process nal tasks is spawned in its own task, we can
                    // put cpu intensive tasks
                    if *self.flir_filter.read().await{
                        self.clone().analyze_image();
                    }
                }
            }
        }
    }
    fn analyze_image(self: Arc<Self>){
        self.clone().handle.spawn_blocking(move ||{
            {
                let mut barrier = self.flir_analysis_barrier.blocking_write();
                if *barrier{
                    println!("NOT Analyizing image");
                    return;
                }
                *barrier = true;
            }
            println!("Analyizing image");

            // Pull in our image
            let image = self.flir_image.blocking_read().clone().into_rgb8();
            let mut filtered_image = DynamicImage::new_rgb8(image.width(), image.height()).into_rgb8();
            let filter_level = *self.flir_filter_level.blocking_read();
            let mut pixels = Vec::with_capacity((image.width() * image.height() * 3) as usize);

            for (x,y,pix) in image.enumerate_pixels().filter(|(_,_,pix)| {pix.0[0] > filter_level}){
                filtered_image.put_pixel(x,y,*pix);
                pixels.push((x,y))
            }

            if let Some(i) = &mut (*self.flir_filtered_image.blocking_write()){
                *i = DynamicImage::from(filtered_image);
            }
        
            if pixels.len() <= 10{
                *self.flir_centroids.blocking_write() = Default::default();
                *self.flir_analysis_barrier.blocking_write() = false;
                return;
            }


            pixels.sort_unstable_by_key(|(_, y)| *y);
            let pix_count = pixels.len();

            let (lpix, upix) = pixels.split_at_mut(pix_count/2);
        
            let lower_y_median = lpix[lpix.len()/2];
            lpix.sort_unstable_by_key(|(x,_)| *x);
            let lower_x_median = lpix[lpix.len()/2];
            let lower_centroid = Vec2::new(lower_x_median.0 as f32, lower_y_median.1 as f32);


            let mut upix = upix.to_vec();
            let mut upper_centroid = Default::default();
            for _ in 0..*self.flir_target_iterations.blocking_read(){
                let upper_y_median = match upix.get(upix.len()/2){
                    Some(p) => *p,
                    None => (0,0),
                };
                upix.sort_unstable_by_key(|(x,_)| *x);
                let upper_x_median = match upix.get(upix.len()/2){
                    Some(p) => *p,
                    None => (0,0),
                };
                upper_centroid = Vec2::new(upper_x_median.0 as f32, upper_y_median.1 as f32);
                
                upix.sort_unstable_by_key(|(_,y)| *y);
                let (_,upper) = upix.split_at(upix.len()/2);
                upix = upper.to_vec();
            }
            
            *self.flir_centroids.blocking_write() = (lower_centroid, upper_centroid);
            *self.flir_analysis_barrier.blocking_write() = false;
        });
    }
}