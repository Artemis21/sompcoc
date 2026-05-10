use crate::{Error, State};
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

mod about;
mod balance;
mod leaderboard;
mod shill;
mod shop;

/// The custom ID of any message component the bot sends. These are URL-encoded.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "T")]
enum CustomId {
    #[serde(rename = "s")]
    ShopPage(shop::ShopPage),

    #[serde(rename = "l")]
    LeaderboardButton(leaderboard::LeaderboardButton),
}

impl CustomId {
    /// Handle a component interaction.
    async fn on_interaction(
        ctx: &serenity::Context,
        interaction: &serenity::ComponentInteraction,
        state: &State,
    ) -> Result<(), Error> {
        let this: Self = serde_urlencoded::from_str(&interaction.data.custom_id)?;
        match this {
            Self::ShopPage(page) => page.on_interaction(ctx, interaction, state).await,
            Self::LeaderboardButton(button) => button.on_interaction(ctx, interaction, state).await,
        }
    }
}

impl std::fmt::Display for CustomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = serde_urlencoded::to_string(self).expect("custom ID serialisation shouldn't fail");
        assert!(id.len() <= 100, "custom ID too long: {id}");
        write!(f, "{id}")
    }
}

/// Parse a usize from a string. This is a workaround for nox/serde_urlencoded#26.
fn parse_usize<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

/// Get the list of commands to register with Discord.
pub fn commands() -> Vec<poise::Command<State, Error>> {
    vec![
        balance::balance(),
        shop::shop(),
        shill::shill(),
        leaderboard::leaderboard(),
        about::about(),
    ]
}

/// Handle the event if it's a component interaction.
pub async fn on_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    state: &State,
) -> Result<(), Error> {
    let serenity::FullEvent::InteractionCreate {
        interaction: serenity::Interaction::Component(event),
    } = event
    else {
        return Ok(());
    };
    CustomId::on_interaction(ctx, event, state).await
}

/// The context type - we use `ApplicationContext` because we don't support
/// prefix commands.
type Context<'a> = poise::ApplicationContext<'a, State, Error>;
