use std::env;
use warp::Filter;
use warp::http::Method;
use tracing_subscriber::fmt::format::FmtSpan;
use crate::errors::return_error;

mod types;
mod store;
mod errors;
mod timeline;
mod routes;
mod date_format;

#[tokio::main]
async fn main() {

    let log_filter = std::env::var("RUST_LOG")
        .unwrap_or_else(
            |_|
                "hardcore-api=info,warp=error".to_owned()
        );

    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    //let db_url = "postgres://postgres:password@localhost:5432/hardcore";
    let db_url : String;
    match env::var("DB_URL") {
	    Ok(value) => db_url = value,
	    Err(e) => panic!("DB_URL is missing: {}", e),
    }

    let store = store::Store::new(db_url.as_str()).await;
    let store_filter = warp::any().map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::GET]);

    let get_timelines = warp::get()
        .and(warp::path("timelines"))
        .and(warp::path::end())
        .and(store_filter)
        .and_then(routes::timeline::get_timelines);



    let routes = get_timelines
        .with(cors)
        .with(warp::trace::request())
        .recover(return_error);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

}
