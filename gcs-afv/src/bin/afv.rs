use clap::Parser;
use gcs_afv::operators::afv_launcher;

#[derive(Parser)]
struct AfvArgs{
    #[arg(short, long)]
    server: bool,
}

fn main(){
    pretty_env_logger::init();
    let args = AfvArgs::parse();
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Could not build tokio runtime");
    runtime.block_on(afv_launcher::launch(!args.server, None));
}