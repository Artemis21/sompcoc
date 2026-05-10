use super::Context;
use crate::{Error, Player, player::Timestamp, show_members};

/// How long in ticks (seconds) players must wait between shilling.
const SHILLING_COOLDOWN: u64 = 30;

/// Tell anyone who will listen that SompCoc is the place to be.
#[poise::command(slash_command)]
pub async fn shill(ctx: Context<'_>) -> Result<(), Error> {
    let now = Timestamp::now();
    let mut player = Player::load(&ctx.data(), now, ctx.author())?;
    let message = if now.ticks_since(player.last_shilled_at) < SHILLING_COOLDOWN {
        let flavour = pick_message(COOLDOWN_MESSAGES);
        let cooldown_ends = player
            .last_shilled_at
            .ticks_later(SHILLING_COOLDOWN)
            .discord_relative();
        format!("{flavour}\n-# You can shill again {cooldown_ends}.")
    } else {
        let luck = rand::random_range(0u32..=5);
        let members = luck * player.total_shill_multiplier();
        player.balance += &members;
        player.last_shilled_at = now;
        let flavour = pick_message(if luck <= 1 {
            BAD_MESSAGES
        } else if luck <= 3 {
            OK_MESSAGES
        } else {
            GOOD_MESSAGES
        });
        format!("{flavour}\n-# You gained **{}**.", show_members(&members))
    };
    player.save(&ctx.data())?;
    ctx.reply(message).await?;
    Ok(())
}

fn pick_message(messages: &'static [&'static str]) -> &'static str {
    messages[rand::random_range(0..messages.len())]
}

const COOLDOWN_MESSAGES: &[&str] = &[
    "Calm down, you're putting people off.",
    "You've said the word \"SompCoc\" so many times it's lost all meaning. Try talking about something else for a change.",
    "You've sent one too many emails about SompCoc recently. Rachel Breward sends you a strongly worded email about misuse of department resources.",
    "You try yet again to convince your friend to come to SompCoc, and she tells you to give it a break. To be fair, you have been overdoing it.",
];
const BAD_MESSAGES: &[&str] = &[
    "You burst into LTA half way through an Andrew Ker lecture to talk about SompCoc. Everyone hates you.",
    "You announce a SompCoc picnic in Port Meadow, but you can't convince anyone to touch grass.",
    "Someone asks if you've been involved in any new societies lately, and you mention SompCoc. They burst out laughing.",
    "You decide you need money to attract members, so you contact a trading firm about sponsorships. They tell you you need members to get a sponsorship.",
];
const OK_MESSAGES: &[&str] = &[
    "Someone in the front row debates your lecturer for ten minutes on whether the proof given would still hold in a universe where cats and dogs are the other way around. You take the opportunity to tell the people around you about SompCoc, which is a welcome distraction.",
    "You hack into Canvas and place a huge banner ad for SompCoc. Unfortunately, the department disables Canvas soon after.",
    "Your revision class gets cancelled two minutes *after* it was due to begin. While everyone is standing around confused, you hand out leaflets for SompCoc.",
];
const GOOD_MESSAGES: &[&str] = &[
    "Your tutor asks what everyone's been up to, and you tell them about all the pizza at SompCoc. Your tutorial partners sign up then and there.",
    "You infiltrate CompSoc and suggest `Raphael'); DROP TABLE members;--` for their name continuation. Emancipated members come crawling to SompCoc.",
    "You reanimate the body of William Archibald Spooner, and convince CompSoc to pay him to promote their brand. He does his thing, you get your members.",
];
