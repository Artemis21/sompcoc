use std::{collections::HashMap, time::UNIX_EPOCH};

use kv::Msgpack;
use num_bigint::BigUint;
use poise::serenity_prelude::{
    self as serenity, FormattedTimestamp, FormattedTimestampStyle, User, UserId,
};
use serde::{Deserialize, Serialize};

use crate::{
    Error, State,
    items::{ItemId, get_item},
};

#[derive(Serialize, Deserialize)]
pub struct Player {
    pub discord_id: UserId,
    pub name: String,
    pub balance: BigUint,
    pub balance_last_updated: Timestamp,
    pub last_shilled_at: Timestamp,
    pub item_counts: HashMap<ItemId, u32>,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Timestamp(u64); // seconds actually elapsed since epoch

impl Timestamp {
    pub const LONG_AGO: Self = Self(0);

    pub fn now() -> Self {
        Self(UNIX_EPOCH.elapsed().unwrap().as_secs())
    }

    pub fn ticks_since(self, historical: Timestamp) -> u64 {
        self.0 - historical.0
    }

    pub fn ticks_later(self, ticks: u64) -> Timestamp {
        Self(self.0 + ticks)
    }

    pub fn to_serenity(self) -> serenity::Timestamp {
        serenity::Timestamp::from_unix_timestamp(self.0 as i64).unwrap()
    }

    pub fn discord_relative(self) -> FormattedTimestamp {
        FormattedTimestamp::new(
            self.to_serenity(),
            Some(FormattedTimestampStyle::RelativeTime),
        )
    }
}

impl Player {
    pub fn load(state: &State, now: Timestamp, user: &User) -> Result<Self, Error> {
        match state.players()?.get(&user.id.get().into())? {
            Some(Msgpack(mut player)) => {
                player.update(now);
                Ok(player)
            }
            None => Ok(Self {
                discord_id: user.id,
                name: user.name.clone(),
                balance: BigUint::ZERO,
                balance_last_updated: now,
                last_shilled_at: Timestamp::LONG_AGO,
                item_counts: HashMap::new(),
            }),
        }
    }

    pub fn income_per_tick(&self) -> BigUint {
        self.item_counts
            .iter()
            .flat_map(|(item_id, count)| Some(get_item(*item_id).income_per_tick? * count))
            .sum()
    }

    pub fn total_shill_multiplier(&self) -> BigUint {
        self.item_counts
            .iter()
            .flat_map(|(item_id, count)| Some(get_item(*item_id).shill_multiplier?.pow(*count)))
            .product()
    }

    pub fn item_count(&self, item: ItemId) -> u32 {
        self.item_counts.get(&item).copied().unwrap_or(0)
    }

    pub fn update(&mut self, current_time: Timestamp) {
        let ticks = current_time.ticks_since(self.balance_last_updated);
        self.balance += self.income_per_tick() * ticks;
        self.balance_last_updated = current_time;
    }

    pub fn save(self, state: &State) -> Result<(), Error> {
        let bucket = state.players()?;
        bucket.set(&self.discord_id.get().into(), &Msgpack(self))?;
        bucket.flush()?;
        Ok(())
    }
}
