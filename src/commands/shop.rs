use std::fmt::Write;

use num_traits::Zero;
use poise::{
    CreateReply,
    serenity_prelude::{
        self as serenity, ComponentInteraction, CreateActionRow, CreateEmbed, CreateEmbedFooter,
        CreateInteractionResponse, CreateInteractionResponseMessage, User,
    },
};
use serde::{Deserialize, Serialize};

use super::{Context, CustomId, parse_usize};
use crate::{
    Error, State, indexed_colour,
    items::{SHOP_ITEMS, get_item},
    player::{Player, Timestamp},
    show_members, show_multiplier,
};

/// Level up your SompCoc membership rates with promotional items from the shop.
#[poise::command(slash_command, subcommand_required, subcommands("browse", "buy"))]
pub async fn shop(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("only subcommands can be invoked")
}

/// Take a gander at the wide variety of membership-promoting methods at your disposal.
#[poise::command(slash_command, rename = "browse")]
async fn browse(ctx: Context<'_>) -> Result<(), Error> {
    let page = ShopPage {
        user: ctx.author().id,
        page: 0,
    };
    ctx.send(
        CreateReply::default()
            .embed(page.render_embed(ctx.data(), ctx.author())?)
            .components(page.render_buttons()),
    )
    .await?;
    Ok(())
}

/// Sacrifice some members to the SompCoc demons in exchange for some form of promotion.
#[poise::command(slash_command, rename = "buy")]
async fn buy(
    ctx: Context<'_>,
    #[description = "The item you'd like to buy"]
    #[autocomplete = "autocomplete_shop_item"]
    item: String,
    #[description = "How many you'd like to buy (default: 1)"]
    #[min = 1]
    #[max = 100]
    count: Option<u32>,
) -> Result<(), Error> {
    let search = item.to_lowercase();
    let mut results = SHOP_ITEMS
        .into_iter()
        .filter_map(|item_id| {
            let item = get_item(*item_id);
            if item.name.to_lowercase().contains(&search) {
                Some(item)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if results.is_empty() {
        ctx.reply(format!(
            "Couldn't find item '{item}'. Use `/shop browse` to see a list of valid item names."
        ))
        .await?;
        return Ok(());
    }
    if results.len() > 1 {
        ctx.reply(format!(
            "Ambiguous item name '{item}'. Use `/shop browse` to see a list of valid item names."
        ))
        .await?;
        return Ok(());
    }
    let item = results.pop().unwrap();
    let mut player = Player::load(&ctx.data(), Timestamp::now(), ctx.author())?;
    let count_to_buy = count.unwrap_or(1);
    let count_owned = player.item_count(item.id);
    if count_to_buy + count_owned > 1 && item.only_one {
        ctx.reply(format!(
            "Don't be greedy. Only **1** {} is available (you already have **{}**).",
            item.name, count_owned,
        ))
        .await?;
        return Ok(());
    }
    let price = (count_owned..count_owned + count_to_buy)
        .map(|n| item.price_of_nth(n))
        .sum();
    if player.balance >= price {
        player.balance -= &price;
        *player.item_counts.entry(item.id).or_insert(0u32) += &count_to_buy;
        ctx.reply(format!(
            "You sacrificed **{}** to purchase **{}**.\n-# Now you have **{}** and +{}/sec.",
            show_members(&price),
            item.show_count(count_to_buy),
            show_members(&player.balance),
            show_members(&player.income_per_tick()),
        ))
        .await?;
    } else {
        ctx.reply(format!(
            "You don't have enough sacrifices for that >:[\n**{}** costs **{}**, but you only have **{}**.",
            item.show_count(count_to_buy),
            show_members(&price),
            show_members(&player.balance)
        )).await?;
    }
    player.save(&ctx.data())?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ShopPage {
    /// The user who is navigating the shop.
    #[serde(rename = "u")]
    user: serenity::UserId,

    /// Which page of the shop to view.
    #[serde(rename = "p", deserialize_with = "parse_usize")]
    page: usize,
}

impl ShopPage {
    const ITEMS_PER_PAGE: usize = 5;

    /// Handle the button to navigate to this page being clicked.
    pub async fn on_interaction(
        self,
        ctx: &serenity::Context,
        interaction: &ComponentInteraction,
        state: &State,
    ) -> Result<(), Error> {
        let response = if interaction.user.id == self.user {
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(self.render_embed(state, &interaction.user)?)
                    .components(self.render_buttons()),
            )
        } else {
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Only the original user can use this button. Try running `/shop browse` yourself.")
                    .ephemeral(true),
            )
        };
        interaction.create_response(ctx, response).await?;
        Ok(())
    }

    fn render_embed(&self, state: &State, user: &User) -> Result<CreateEmbed, Error> {
        let player = Player::load(state, Timestamp::now(), user)?;
        let title = format!(
            "Advertisement Broker (page {} of {})",
            self.page + 1,
            SHOP_ITEMS.len().div_ceil(Self::ITEMS_PER_PAGE)
        );
        let fields = SHOP_ITEMS
            .into_iter()
            .skip(self.page * Self::ITEMS_PER_PAGE)
            .take(Self::ITEMS_PER_PAGE)
            .map(|item_id| {
                let item = get_item(*item_id);
                let count = player.item_count(*item_id);
                let title = format!(
                    "{} [{}]",
                    item.name,
                    show_members(&item.price_of_nth(count))
                );
                let mut body = item.description.to_string();
                if item.only_one {
                    body.push_str(" Only **1** available.");
                }
                if let Some(income) = &item.income_per_tick {
                    write!(body, " **{}/sec**", show_members(income)).unwrap();
                }
                if let Some(mult) = &item.shill_multiplier {
                    write!(body, " **{} members from shilling**", show_multiplier(mult)).unwrap();
                }
                if item.only_one && count > 0 {
                    body = format!("~~{body}~~");
                }
                if !count.is_zero() {
                    if item.only_one {
                        write!(body, "\n-# You already own {}", item.name).unwrap();
                    } else {
                        write!(body, "\n-# You own: {}", item.show_count(count)).unwrap();
                    }
                };
                (title, body, false)
            });
        Ok(CreateEmbed::new()
            .title(title)
            .fields(fields)
            .colour(indexed_colour(self.page))
            .footer(CreateEmbedFooter::new(
                "Purchase an item with `/shop buy <item name>`. Items get more expensive the more you have.",
            ))
            .description(
                format!("Sacrifice some of your **{}** to us in exchange for the promotions below...", show_members(&player.balance)),
            ))
    }

    fn render_buttons(&self) -> Vec<CreateActionRow> {
        let prev = CustomId::ShopPage(Self {
            user: self.user,
            page: self.page.saturating_sub(1),
        });
        let next = CustomId::ShopPage(Self {
            user: self.user,
            page: self.page + 1,
        });
        let buttons = vec![
            serenity::CreateButton::new(prev.to_string())
                .emoji('⬅')
                .disabled(self.page == 0),
            serenity::CreateButton::new(next.to_string())
                .emoji('➡')
                .disabled(SHOP_ITEMS.len() <= (self.page + 1) * Self::ITEMS_PER_PAGE),
        ];
        vec![serenity::CreateActionRow::Buttons(buttons)]
    }
}

async fn autocomplete_shop_item(_ctx: Context<'_>, partial: &str) -> Vec<&'static str> {
    let search = partial.to_lowercase();
    SHOP_ITEMS
        .into_iter()
        .filter_map(|item_id| {
            let item = get_item(*item_id);
            if item.name.to_lowercase().contains(&search) {
                Some(item.name)
            } else {
                None
            }
        })
        .take(25)
        .collect()
}
