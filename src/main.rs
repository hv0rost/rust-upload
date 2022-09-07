mod server;
mod utils;
pub(crate) mod data_base;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    data_base::connection::create_connection_pool().await;
    server::start(([127,0,0,1], 3030)).await;
}
