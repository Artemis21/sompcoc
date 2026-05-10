use kv::{Batch, Integer, Msgpack};
use num_bigint::{BigInt, BigUint};
use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};

use crate::{Error, State, player::Timestamp};

pub const ENTRIES_PER_PAGE: usize = 15;
pub const REFRESH_FREQUENCY: u64 = 60;

#[derive(Clone, Deserialize, Serialize)]
pub struct LeaderboardPage {
    pub generated_at: Timestamp,
    pub entries: Vec<LeaderboardEntry>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct LeaderboardEntry {
    pub position: usize,
    pub discord_id: UserId,
    pub name: String,
    pub balance: BigUint,
    pub income: BigUint,
}

pub fn get_page(state: &State, page: usize) -> Result<LeaderboardPage, Error> {
    ensure_fresh_leaderboard(state)?;
    Ok(state.leaderboard()?.get(&Integer::from(page))?.unwrap().0)
}

pub fn get_page_count(state: &State) -> Result<usize, Error> {
    ensure_fresh_leaderboard(state)?;
    Ok(state.leaderboard()?.len())
}

pub fn ensure_fresh_leaderboard(state: &State) -> Result<(), Error> {
    let now = Timestamp::now();
    if now.ticks_since(leaderboard_last_updated(state)?) > REFRESH_FREQUENCY {
        rebuild_leaderboard(state, now)?;
    }
    Ok(())
}

fn leaderboard_last_updated(state: &State) -> Result<Timestamp, Error> {
    Ok(state
        .leaderboard()?
        .get(&Integer::from(0))?
        .map_or(Timestamp::LONG_AGO, |Msgpack(page)| page.generated_at))
}

fn rebuild_leaderboard(state: &State, now: Timestamp) -> Result<(), Error> {
    let mut entries = state
        .players()?
        .iter()
        .map(|item| {
            let Msgpack(mut player) = item?.value()?;
            player.update(now);
            Ok(LeaderboardEntry {
                position: 0,
                discord_id: player.discord_id,
                name: player.name.clone(),
                balance: player.balance.clone(),
                income: player.income_per_tick(),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    entries.sort_by_key(|entry| BigInt::from(entry.balance.clone()) * -1);
    for (i, entry) in entries.iter_mut().enumerate() {
        entry.position = i;
    }
    let mut batch = Batch::new();
    for (i, page_entries) in entries.chunks(ENTRIES_PER_PAGE).enumerate() {
        batch.set(
            &Integer::from(i),
            &Msgpack(LeaderboardPage {
                generated_at: now,
                entries: page_entries.iter().cloned().collect(),
            }),
        )?;
    }
    if entries.is_empty() {
        batch.set(
            &Integer::from(0),
            &Msgpack(LeaderboardPage {
                generated_at: now,
                entries: vec![],
            }),
        )?;
    }
    let leaderboard = state.leaderboard()?;
    leaderboard.clear()?;
    leaderboard.batch(batch)?;
    Ok(())
}
