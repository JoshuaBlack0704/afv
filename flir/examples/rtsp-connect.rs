use eframe::egui;
use futures::StreamExt;
use retina::client::{SessionOptions, self, SetupOptions, PlayOptions};
use url::Url;

fn main(){
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("My egui App", native_options, Box::new(|cc| Box::new(MyEguiApp::new(cc))));
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    runtime.block_on(stream());


}

async fn stream()
{
        let url = Url::parse(&format!("rtsp://:@172.20.10.9:554/avc/ch1")).expect("Faulty ip addr");
        let mut options = SessionOptions::default();
        options = options.user_agent(String::from("Flir"));

        let mut session = client::Session::describe(url, options).await.expect("Could not establish session with A50");
        let options = SetupOptions::default();
        session.setup(0, options).await.expect("Could not initiate stream with A50");
        let options = PlayOptions::default();
        let err = format!("Could not start playing string {}", 0);
        let play = session.play(options).await.expect(&err);
        let mut play = play.demuxed().expect("Could not demux the playing stream");
        while let Some(item) = play.next().await{
            match item{
                Ok(_) => {
                
            },
                Err(_) => todo!(),
            }
        }
    
}

#[derive(Default)]
struct MyEguiApp {}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
       egui::CentralPanel::default().show(ctx, |ui| {
           ui.heading("Hello World!");
       });
   }
}
