use std::net::SocketAddr;

mod routes;

pub async fn start(addr : impl Into<SocketAddr>){
    warp::serve(routes::make_routes().await).run(addr).await;
}
