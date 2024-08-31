use crate::store::Store;
use crate::timeline;
use crate::timeline::Timeline;
use tracing::{event, instrument, Level};

#[instrument]
pub async fn get_timelines(store: Store) -> Result<impl warp::Reply, warp::Rejection> {

    event!(target: "hardcore-api", Level::INFO, "loading timelines");

    let players = match store.get_players().await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };
    let mut timelines: Vec<Timeline> = vec![];

    for player in players {
        let timeline = match timeline::Timeline::build(&store, &player).await {
            Ok(r) => r,
            Err(e) => return Err(warp::reject::custom(e)),
        };
        //print!("{:?}", timeline);
        timelines.push(timeline)
    }

    timelines.sort();
    timelines.reverse();

    Ok(warp::reply::json(&timelines))
}
