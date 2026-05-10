use super::Context;
use crate::{
    Error, Player,
    items::{SHOP_ITEMS, get_item},
    player::Timestamp,
    random_colour, show_members, show_multiplier,
};
use poise::serenity_prelude as serenity;

/// Retrieve membership stats for your personal SompCoc.
#[poise::command(slash_command, rename = "membership")]
pub async fn balance(ctx: Context<'_>) -> Result<(), Error> {
    let player = Player::load(&ctx.data(), Timestamp::now(), ctx.author())?;
    let mut items_summary = String::new();
    for item_id in SHOP_ITEMS {
        if let Some(count) = player.item_counts.get(item_id)
            && *count > 0
        {
            let item = get_item(*item_id);
            if !items_summary.is_empty() {
                items_summary.push_str(" • ");
            }
            items_summary.push_str(&item.show_count(*count))
        }
    }
    if items_summary.is_empty() {
        items_summary.push_str("Nothing yet `._.` Check out `/shop browse`.")
    }
    let embed = serenity::CreateEmbed::new()
        .title(format!("{} SompCoc Membership", player.name))
        .description(format!(
            "**{}** +{}/sec; {} shill multiplier",
            show_members(&player.balance),
            show_members(&player.income_per_tick()),
            show_multiplier(&player.total_shill_multiplier()),
        ))
        .field("Your Advertisements", items_summary, false)
        .colour(random_colour());
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
