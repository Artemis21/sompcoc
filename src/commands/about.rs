use super::Context;
use crate::{Error, random_colour};
use poise::serenity_prelude as serenity;

/// OwO, what's this?
#[poise::command(slash_command)]
pub async fn about(ctx: Context<'_>) -> Result<(), Error> {
    let embed = serenity::CreateEmbed::new()
        .title("SompCoc Rises")
        .description(
            "For many years, CompSoc has reigned over a quiet but bountiful UGSA, while the hitherto mysterious SompCoc has rested idly in the shadows. Now, it's your turn to take control: by any means available, capture the hearts and minds of every undergraduate in the department, and displace CompSoc from the throne.\n\nStart out with `/shill` to pick up your first few members, then go to `/shop browse` to automate the process like a true CSser. Check out your progress with `/membership` and see how others are doing with `/leaderboard`."
        )
        .field("Author", "Miriam (amv21)", true)
        .field("Library", "serenity-rs", true)
        .colour(random_colour());
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
