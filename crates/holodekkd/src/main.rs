use clap::Parser;

use holodekk::api;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Options {
    /// Port to listen on
    #[arg(long, short, default_value = "6080")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let options = Options::parse();

    api::server::run(options.port.to_owned()).await;
}
