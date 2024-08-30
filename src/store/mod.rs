use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use sqlx::postgres::{PgPoolOptions, PgRow};
use uuid::Uuid;
use crate::errors::Error;
use crate::types::death::Death;
use crate::types::extra_life::ExtraLife;
use crate::types::offense::Offense;
use crate::types::player::Player;

#[derive(Clone, Debug)]
pub struct Store {
    connection: PgPool,
}

impl Store {
    pub async fn new(db_url: &str) -> Self {
        let db_pool = match PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url).await {
            Ok(pool) => pool,
            Err(e) => panic!("Couldn't establish DB connection:{}", e),
        };

        Store {
            connection: db_pool,
        }
    }

    pub async fn get_players(&self) -> Result<Vec<Player>, Error> {
        match sqlx::query("SELECT player_uuid, name, joined, playtime FROM players")
            .map(|row: PgRow| {
                let u : uuid::Uuid = row.get(0);
                //let unix_timestamp: i32 = row.get(2);
                //let t = DateTime::<Utc>::from_timestamp(unix_timestamp as i64, 0).unwrap();
                let t: DateTime<Utc> = row.get(2);

                Player {
                    player_uuid: u.to_string(),
                    name: row.get(1),
                    joined: t,
                    playtime: row.get(3),
                }
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(players) => Ok(players),
            Err(e) => {
                // todo -p print
                println!("{}", e);
                Err(Error::DatabaseQueryError)
            },
        }
    }

    pub async fn get_player_deaths(&self, player_uuid: &String) -> Result<Vec<Death>, Error> {
        match sqlx::query("SELECT id, player_uuid, stamp, playtime, reason FROM deaths WHERE player_uuid = $1")
            .bind(Uuid::parse_str(player_uuid).unwrap())
            .map(|row: PgRow| {
                let u : uuid::Uuid = row.get(1);
                //let unix_timestamp: i32 = row.get(2);
                //let t = DateTime::<Utc>::from_timestamp(unix_timestamp as i64, 0).unwrap();
                let t: DateTime<Utc> = row.get(2);

                Death {
                    id: row.get(0),
                    player_uuid: u.to_string(),
                    stamp: t,
                    playtime: row.get(3),
                    reason: row.get(4),
                }
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(deaths) => Ok(deaths),
            Err(e) => {
                // todo -p print
                println!("{}", e);
                Err(Error::DatabaseQueryError)
            },
        }
    }

    pub async fn get_player_extra_lives(&self, player_uuid: &String) -> Result<Vec<ExtraLife>, Error> {
        match sqlx::query("SELECT id, player_uuid, stamp, reason, playtime FROM extra_lives WHERE player_uuid = $1")
            .bind(Uuid::parse_str(player_uuid).unwrap())
            .map(|row: PgRow| {
                let u : uuid::Uuid = row.get(1);
                //let unix_timestamp: i32 = row.get(2);
                //let t = DateTime::<Utc>::from_timestamp(unix_timestamp as i64, 0).unwrap();
                let t: DateTime<Utc> = row.get(2);

                ExtraLife {
                    id: row.get(0),
                    player_uuid: u.to_string(),
                    stamp: t,
                    reason: row.get(3),
                    playtime: row.get(4),
                }
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(extra_lives) => Ok(extra_lives),
            Err(e) => {
                // todo -p print
                println!("{}", e);
                Err(Error::DatabaseQueryError)
            },
        }
    }

    pub async fn get_player_offenses(&self, player_uuid: &String) -> Result<Vec<Offense>, Error> {
        match sqlx::query("SELECT id, player_uuid, stamp, reason, playtime FROM offenses WHERE player_uuid = $1")
            .bind(Uuid::parse_str(player_uuid).unwrap())
            .map(|row: PgRow| {
                let u : uuid::Uuid = row.get(1);
                //let unix_timestamp: i32 = row.get(2);
                //let t = DateTime::<Utc>::from_timestamp(unix_timestamp as i64, 0).unwrap();
                let t: DateTime<Utc> = row.get(2);

                Offense {
                    id: row.get(0),
                    player_uuid: u.to_string(),
                    stamp: t,
                    reason: row.get(3),
                    playtime: row.get(4),
                }
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(offenses) => Ok(offenses),
            Err(e) => {
                // todo -p print
                println!("{}", e);
                Err(Error::DatabaseQueryError)
            },
        }
    }
}