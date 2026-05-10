use kv::{Bucket, Store};
use num_bigint::BigUint;
use num_format::{Locale, ToFormattedString};
use poise::serenity_prelude as serenity;

use crate::{leaderboard::LeaderboardPage, player::Player};

mod commands;
mod items;
mod leaderboard;
mod player;

struct State {
    pub kv: Store,
}

impl State {
    pub fn players(&self) -> Result<Bucket<'_, kv::Integer, kv::Msgpack<Player>>, Error> {
        Ok(self.kv.bucket(Some("players"))?)
    }

    pub fn leaderboard(
        &self,
    ) -> Result<Bucket<'_, kv::Integer, kv::Msgpack<LeaderboardPage>>, Error> {
        Ok(self.kv.bucket(Some("leaderboard"))?)
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv()?;
    pretty_env_logger::init();
    let token = dotenvy::var("DISCORD_TOKEN")?;
    let guild_id = dotenvy::var("GUILD_ID")?.parse()?;
    let kv_path: String = dotenvy::var("KV_PATH")?.parse()?;
    let opts = poise::FrameworkOptions {
        commands: commands::commands(),
        event_handler: |ctx, ev, _, state| Box::pin(commands::on_event(ctx, ev, state)),
        ..Default::default()
    };
    let state = State {
        kv: Store::new(kv::Config::new(kv_path))?,
    };
    let framework = poise::Framework::builder()
        .setup(move |ctx, _, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id)
                    .await?;
                Ok(state)
            })
        })
        .options(opts)
        .build();
    let intents = serenity::GatewayIntents::empty();
    serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap()
        .start()
        .await
        .unwrap();
    Ok(())
}

const COLOURS: &[u32] = &[0xff4a8e, 0x8a6fd5, 0x00c5ba];

fn random_colour() -> u32 {
    COLOURS[rand::random_range(0..COLOURS.len())]
}

fn indexed_colour(idx: usize) -> u32 {
    COLOURS[idx % COLOURS.len()]
}

fn show_members(num: &BigUint) -> String {
    if num == &BigUint::from(1u32) {
        "1 member".to_string()
    } else {
        format!("{} members", num.to_formatted_string(&Locale::en))
    }
}

fn show_multiplier(num: &BigUint) -> String {
    format!("×{}", num.to_formatted_string(&Locale::en))
}
