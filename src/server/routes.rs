use mime::Mime;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

use crate::utils::deduplication;

pub(super) async fn make_routes() -> BoxedFilter<(impl Reply,)>{
    let upload = warp::any()
        .and(warp::header::<Mime>("content-type"))
        .and(warp::body::stream())
        .and_then(deduplication::multi_part);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST"])
        .allow_headers(vec!["content-type", "Access-Control-Request-Method", "Access-Control-Request-Headers", "Authorization"]);

    upload
        .with(cors)
        .boxed()
}


