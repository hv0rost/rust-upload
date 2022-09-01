mod server;
mod utils;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    server::start(([127,0,0,1], 3030)).await;
}
