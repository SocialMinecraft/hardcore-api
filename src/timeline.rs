use std::cmp::Ordering;
use chrono::Utc;
//use serde::de::Unexpected::Option;
use serde::Serialize;
use crate::errors::Error;
use crate::store::Store;
use crate::types::death::Death;
use crate::types::extra_life::ExtraLife;
use crate::types::offense::Offense;
use crate::types::player::Player;

#[derive(Serialize, Debug, Eq, PartialEq)]
pub struct Timeline {
    pub player_name: String,
    pub player_state: PlayerState,
    pub survived_seconds: i32,
    pub events: Vec<TimelineEvent>,
    pub longest_life_seconds: i32,
    pub shortest_life_seconds: i32,
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

#[derive(Serialize, Debug, PartialEq, Eq)]
pub enum EventType {
    Joined,
    Died,
    ExtraLife,
    Offense,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct TimelineEvent {
    pub stamp : chrono::DateTime<Utc>,
    pub what : EventType,
    pub context: String,
    pub playtime: i32,
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
        let mut deaths = store.get_player_deaths(&player.player_uuid).await?;
        let offenses = store.get_player_offenses(&player.player_uuid).await?;
        let extra_lives = store.get_player_extra_lives(&player.player_uuid).await ?;

	deaths.sort(); // sorting for the longest / shortest later.

        let player_state = if deaths.len() - extra_lives.len() >= 3 { PlayerState::Dead } else { PlayerState::Alive };

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
        });
        events.sort();

        let mut survived_seconds = player.playtime.clone();
        if player_state == PlayerState::Dead {
            let last_death = deaths.iter().max(); // this is sorted, so I coul just grab the last.
            if last_death.is_some() {
                survived_seconds = last_death.unwrap().playtime;
            }
        }
	
        // normalize ticks to seconds
        survived_seconds /= 20;

	let (long, short) = Self::longest_shortest_life(&deaths, &player.playtime, player_state == PlayerState::Alive);

        Ok(Timeline {
            player_name: player.name.clone(),
            player_state,
            events,
            survived_seconds,
            longest_life_seconds: long / 20,
            shortest_life_seconds: short / 20,
        })
    }

    // Todo - need to consider time a player may spent as a ghost.
    fn longest_shortest_life(deaths: &Vec<Death>, playtime: &i32, alive: bool) -> (i32, i32) {

        //let mut time_between = playtime.clone();
        let mut last_playtime = 0;
        let mut long = 0;
        let mut short = playtime.clone();

        for death in deaths {
            let time_between = death.playtime - last_playtime;
            last_playtime = death.playtime.clone();

            if time_between < short {
                short = time_between.clone();
            }
            if time_between > long {
                long = time_between.clone();
            }
        }

        if alive {
            let time_between = playtime - last_playtime;

            if time_between > 0 && time_between < short {
                short = time_between.clone();
            }
            if time_between > 0 && time_between > long {
                long = time_between.clone();
            }
        }

        (long, short)
    }

    fn death_to_event(a: &Death) -> TimelineEvent {
        TimelineEvent {
            stamp: a.stamp.clone(),
            what: EventType::Died,
            context: a.reason.clone(),
            playtime: a.playtime / 20,
        }
    }

    fn offense_to_event(a: &Offense) -> TimelineEvent {
        TimelineEvent {
            stamp: a.stamp.clone(),
            what: EventType::Offense,
            context: a.reason.clone(),
            playtime: a.playtime / 20,
        }
    }

    fn extra_life_to_event(a: &ExtraLife) -> TimelineEvent {
        TimelineEvent {
            stamp: a.stamp.clone(),
            what: EventType::ExtraLife,
            context: a.reason.clone(),
            playtime: a.playtime / 20,
        }
    }
}
