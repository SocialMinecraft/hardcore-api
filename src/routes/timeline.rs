use crate::store::Store;
use crate::timeline;
use crate::timeline::Timeline;
use tracing::{event, instrument, Level};

#[instrument]
pub async fn get_timelines(store: Store) -> Result<impl warp::Reply, warp::Rejection> {

    event!(target: "hardcore-api", Level::INFO, "loading timelines");

    // todo - handle the errors....
    // Err(warp::reject::custom(e))

    let players = store.get_players().await.unwrap();
    let mut timelines: Vec<Timeline> = vec![];

    for player in players {
        let timeline = timeline::Timeline::build(&store, &player).await.unwrap();
        //print!("{:?}", timeline);
        timelines.push(timeline)
    }

    timelines.sort();
    timelines.reverse();

    Ok(warp::reply::json(&timelines))
}
