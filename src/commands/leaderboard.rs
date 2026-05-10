use std::fmt::Write;

use poise::{
    CreateReply,
    serenity_prelude::{
        self as serenity, ComponentInteraction, CreateActionRow, CreateEmbed, CreateEmbedFooter,
        CreateInteractionResponse, CreateInteractionResponseMessage,
    },
};
use serde::{Deserialize, Serialize};

use super::{Context, CustomId, parse_usize};
use crate::{
    Error, State, indexed_colour,
    leaderboard::{REFRESH_FREQUENCY, get_page, get_page_count},
    show_members,
};

/// See who is running the best SompCoc.
#[poise::command(slash_command)]
pub async fn leaderboard(
    ctx: Context<'_>,
    #[description = "Which page of the leaderboard to start from (default: 1)"]
    #[min = 1]
    page: Option<usize>,
) -> Result<(), Error> {
    let mut page = LeaderboardButton {
        user: ctx.author().id,
        page: page.unwrap_or(1),
    };
    page.limit_page(ctx.data())?;
    ctx.send(
        CreateReply::default()
            .embed(page.render_embed(ctx.data())?)
            .components(page.render_buttons(ctx.data())?),
    )
    .await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct LeaderboardButton {
    /// The user who is navigating the leaderboard.
    #[serde(rename = "u")]
    user: serenity::UserId,

    /// Which page of the leaderboard to view.
    #[serde(rename = "p", deserialize_with = "parse_usize")]
    page: usize,
}

impl LeaderboardButton {
    /// Handle the button to navigate to this page being clicked.
    pub async fn on_interaction(
        mut self,
        ctx: &serenity::Context,
        interaction: &ComponentInteraction,
        state: &State,
    ) -> Result<(), Error> {
        self.limit_page(state)?;
        let response = if interaction.user.id == self.user {
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(self.render_embed(state)?)
                    .components(self.render_buttons(state)?),
            )
        } else {
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Only the original user can use this button. Try running `/leaderboard` yourself.")
                    .ephemeral(true),
            )
        };
        interaction.create_response(ctx, response).await?;
        Ok(())
    }

    fn limit_page(&mut self, state: &State) -> Result<(), Error> {
        let page_count = get_page_count(state)?;
        if self.page >= page_count {
            self.page = page_count - 1;
        }
        Ok(())
    }

    fn render_embed(&self, state: &State) -> Result<CreateEmbed, Error> {
        let page = get_page(state, self.page)?;
        let title = format!(
            "Best SompCoc Presidents (page {} of {})",
            self.page + 1,
            get_page_count(state)?,
        );
        let mut body = String::new();
        for entry in page.entries {
            write!(
                &mut body,
                "{}. **{}**: {}\n",
                entry.position + 1,
                entry.name,
                show_members(&entry.balance)
            )?;
        }
        if body.is_empty() {
            body.push_str("`o_o`\n-# There's nothing here. What are you doing?");
        }
        Ok(CreateEmbed::new()
            .title(title)
            .description(body)
            .colour(indexed_colour(self.page))
            .footer(CreateEmbedFooter::new(format!(
                "The leaderboard updates every {} seconds. Last update: ",
                REFRESH_FREQUENCY
            )))
            .timestamp(page.generated_at.to_serenity()))
    }

    fn render_buttons(&self, state: &State) -> Result<Vec<CreateActionRow>, Error> {
        let page_count = get_page_count(state)?;
        if page_count == 1 {
            return Ok(vec![]);
        }
        let prev = CustomId::LeaderboardButton(Self {
            user: self.user,
            page: self.page.saturating_sub(1),
        });
        let next = CustomId::LeaderboardButton(Self {
            user: self.user,
            page: self.page + 1,
        });
        let buttons = vec![
            serenity::CreateButton::new(prev.to_string())
                .emoji('⬅')
                .disabled(self.page == 0),
            serenity::CreateButton::new(next.to_string())
                .emoji('➡')
                .disabled(self.page >= page_count - 1)
        ];
        Ok(vec![serenity::CreateActionRow::Buttons(buttons)])
    }
}
