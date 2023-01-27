use std::net::SocketAddr;

use retina::{self, client::{self, SessionOptions, SetupOptions, PlayOptions, Demuxed}, codec::CodecItem};
use futures::{stream::Stream, StreamExt};
use url::Url;
pub struct A50{
}

impl A50{
    pub async fn new(addr: SocketAddr){
        let url = Url::parse(&format!("rtsp://{}/", &addr)).expect("Faulty ip addr");
        let mut options = SessionOptions::default();
        options = options.user_agent(String::from("Flir"));

        let mut session = client::Session::describe(url, options).await.expect("Could not establish session with A50");
        let options = SetupOptions::default();
        session.setup(0, options).await.expect("Could not initiate stream with A50");
        let options = PlayOptions::default();
        let err = format!("Could not start playing string {}", 0);
        let play = session.play(options).await.expect(&err);
        let demux = play.demuxed().expect("Could not demux the playing stream");
        tokio::pin!(demux);
        while let Some(item) = demux.next().await{
            match item{
                Ok(e) => println!("Recieved frame"),
                Err(_) => todo!(),
            }
        }


    }
}