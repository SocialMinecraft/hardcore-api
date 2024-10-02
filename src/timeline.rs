use std::cmp::Ordering;
use std::option::Option;
use chrono::Utc;
use serde::Serialize;
use crate::errors::Error;
use crate::store::Store;
use crate::types::death::Death;
use crate::types::extra_life::ExtraLife;
use crate::types::offense::Offense;
use crate::types::player::Player;
use crate::date_format;
use tracing::{event, Level};

#[derive(Serialize, Debug, Eq, PartialEq)]
pub struct Timeline {
    pub player_name: String,
    pub player_state: PlayerState,
    pub survived_seconds: i32,
    pub longest_life_seconds: i32,
    pub shortest_life_seconds: i32,
    pub events: Vec<TimelineEvent>,
}

impl PartialOrd for Timeline {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.survived_seconds.partial_cmp(&other.survived_seconds)
    }
}

impl Ord for Timeline {
    fn cmp(&self, other: &Self) -> Ordering {
        self.survived_seconds.cmp(&other.survived_seconds)
    }
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub enum PlayerState {
    Alive,
    Dead,
    //Unranked,
}

#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub enum EventType {
    Joined,
    Died,
    ExtraLife,
    Offense,
    Alive,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct TimelineEvent {
    #[serde(with = "date_format")]
    pub stamp : chrono::DateTime<Utc>,
    pub what : EventType,
    pub context: String,
    pub playtime: i32,

    /// Only on a death and alive event, how many ticks was the player alive
    pub span: i32,
    /// Only useful on a death and alive event and extra life, is the life unranked.
    pub unranked: bool,
}

impl PartialOrd for TimelineEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.stamp.partial_cmp(&other.stamp)
    }
}

impl Ord for TimelineEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.stamp.cmp(&other.stamp)
    }
}

impl Timeline {

    pub async fn build(store: &Store, player: &Player) -> Result<Self, Error> {
        let deaths = store.get_player_deaths(&player.player_uuid).await?;
        let offenses = store.get_player_offenses(&player.player_uuid).await?;
        let extra_lives = store.get_player_extra_lives(&player.player_uuid).await ?;

        let player_state = if deaths.len() >= extra_lives.len() + 3  { PlayerState::Dead } else { PlayerState::Alive };

        let mut events : Vec<TimelineEvent> = Vec::new();
        for death in &deaths {
            events.push(Self::death_to_event(death))
        }
        for offense in &offenses {
            events.push(Self::offense_to_event(offense))
        }
        for extra_life in &extra_lives {
            events.push(Self::extra_life_to_event(extra_life))
        }
        events.push(TimelineEvent{
            stamp: player.joined.clone(),
            what: EventType::Joined,
            context: "Joined Hardcore".to_string(),
            playtime: 0,
            span: 0,
            unranked: false,
        });
        if player_state == PlayerState::Alive {
            events.push(TimelineEvent{
                stamp: Utc::now(),
                what: EventType::Alive,
                context: "Player is alive".to_string(),
                playtime: player.playtime,
                span: 0,
                unranked: false,
            });
        }

        Self::calculate_spans(&mut events);
        let (long, short, survived) = Self::find_meta_stats(&events);
        Self::normalize_event_spans(&mut events);

        Ok(Timeline {
            player_name: player.name.clone(),
            player_state,
            events,
            survived_seconds: survived / 20, // normalize to seconds
            longest_life_seconds: long / 20, // normalize to seconds
            shortest_life_seconds: short / 20, // normalize to seconds
        })
    }

    /// Will convert the spans to seconds from ticks. Should call this at the end
    /// as you will lose some data.
    fn normalize_event_spans(events: &mut Vec<TimelineEvent>) {
        for event in events {
            event.span /= 20;
            event.playtime /= 20;
        }
    }

    // events must be ordered before this point.
    fn calculate_spans(events: &mut Vec<TimelineEvent>) {

        events.sort();

        let mut prev_playtime = 0;
        let mut ranked_lives = 3;
        let mut unranked_lives = 0;

        for event in events {

            match event.what {
                EventType::Joined => { /* Nothing to do */ },
                EventType::Died => {
                    let ranked;
                    // remove life.
                    if ranked_lives > 0 {
                        ranked_lives -= 1;
                        ranked = true;
                    } else if unranked_lives > 0 {
                        unranked_lives -=1;
                        ranked = false;
                    } else {
                        event!(target: "hardcore-api", Level::WARN, "Player has no lives but died.");
                        event.span = 20*60*60*24*7*-1;
                        event.unranked = true;
                        continue;
                    }

                    // how long did the player live.
                    event.span = event.playtime - prev_playtime;

                    // unranked life?
                    event.unranked = !ranked;

                    // set next playtime
                    prev_playtime = event.playtime.clone();
                },
                EventType::Alive => {
                    // how long did the player live.
                    event.span = event.playtime - prev_playtime;

                    // unranked life?
                    event.unranked = ranked_lives <= 0;
                }
                EventType::Offense => { /* Nothing to do */ },
                EventType::ExtraLife => {
                    // if the playtime is zero, we don't have data so we can't update playtime
                    // is the player a ghost when they got the extra life?
                    if event.playtime != 0  &&  ranked_lives + unranked_lives <= 0 {
                        // set the playtime to their current time.
                        prev_playtime = event.playtime.clone();
                    }

                    // Is this a paid life?
                    if event.context == "PAID" {
                        unranked_lives += 1;
                        //next_unranked = true;
                        event.unranked = true;
                    } else {
                        ranked_lives += 1;
                    }
                },
            };
        }
    }


    /// This function returns the longest life, shortest  life, and total survive time.
    /// ignores any unranked spanns.
    fn find_meta_stats(events: &Vec<TimelineEvent>) -> (i32, i32, i32) {

        let mut long = 0;
        let mut short = i32::MAX;
        let mut survive_time = 0;

        for event in events {
            if event.what != EventType::Died && event.what != EventType::Alive {
                continue;
            }

            if event.unranked {
                continue;
            }

            survive_time += event.span;

            if event.span > long {
                long = event.span.clone();
            }
            if event.span < short {
                short = event.span.clone();
            }
        }

        (long, short, survive_time)
    }

    fn death_to_event(a: &Death) -> TimelineEvent {
        TimelineEvent {
            stamp: a.stamp.clone(),
            what: EventType::Died,
            context: a.reason.clone(),
            playtime: a.playtime,
            span: 0,
            unranked: false,
        }
    }

    fn offense_to_event(a: &Offense) -> TimelineEvent {
        TimelineEvent {
            stamp: a.stamp.clone(),
            what: EventType::Offense,
            context: a.reason.clone(),
            playtime: a.playtime,
            span: 0,
            unranked: false,
        }
    }

    fn extra_life_to_event(a: &ExtraLife) -> TimelineEvent {
        TimelineEvent {
            stamp: a.stamp.clone(),
            what: EventType::ExtraLife,
            context: a.reason.clone(),
            playtime: a.playtime,
            span: 0,
            unranked: false,
        }
    }
}
