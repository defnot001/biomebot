use std::fmt::Display;

use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{CreateAttachment, Embed, ExecuteWebhook, Webhook};

use crate::{respond_error, respond_mistake, util::embeds::EmbedColor, Context};

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
#[repr(u8)]
pub enum TargetChannelWebhook {
    Rules,
    Roles,
}

impl Display for TargetChannelWebhook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Roles => write!(f, "roles"),
            Self::Rules => write!(f, "rules"),
        }
    }
}

/// Subcommands for manual embed creation.
#[poise::command(
    slash_command,
    guild_only = true,
    default_member_permissions = "ADMINISTRATOR",
    subcommands("simple", "custom", "example"),
    subcommand_required
)]
pub async fn embed(_: Context<'_>) -> anyhow::Result<()> {
    Ok(())
}

/// Simple with text content and a title.
#[poise::command(slash_command, guild_only = true)]
pub async fn simple(
    ctx: Context<'_>,
    #[description = "The target channel webhook for your embed to go."]
    channel: TargetChannelWebhook,
    #[description = "The description (text) of the embed."] content: String,
    #[description = "The title of the embed."] title: Option<String>,
    #[description = "The color of the embed side."] colour: Option<EmbedColor>,
) -> anyhow::Result<()> {
    ctx.defer_ephemeral().await?;

    let webhook = Webhook::from_url(&ctx, ctx.data().config.webhook_url(channel)).await?;

    let mut embed = Embed::default();
    embed.kind = Some("rich".into());
    embed.description = Some(content);
    embed.title = title;
    embed.colour = Some(colour.unwrap_or_default().into());

    match webhook
        .execute(&ctx, false, ExecuteWebhook::new().embed(embed.into()))
        .await
    {
        Ok(_) => {
            ctx.say(format!("Successfully posted embed in {channel} channel."))
                .await?;
        }
        Err(e) => {
            respond_error!("Failed to post embed in {channel} channel", e, &ctx);
        }
    }

    Ok(())
}

/// Pass in a json object to to send a custom embed.
#[poise::command(slash_command, guild_only = true)]
pub async fn custom(
    ctx: Context<'_>,
    #[description = "The target channel webhook for your embed to go."]
    channel: TargetChannelWebhook,
    #[description = "The json representation of the embed you want to post."] content: String,
) -> anyhow::Result<()> {
    ctx.defer_ephemeral().await?;

    let webhook = Webhook::from_url(&ctx, ctx.data().config.webhook_url(channel)).await?;

    let mut embed = match serde_json::from_str::<Embed>(&content) {
        Ok(json) => json,
        Err(e) => {
            respond_error!("Failed to parse the provided json", e, &ctx);
        }
    };

    embed.kind = Some("rich".into());

    if embed.description.is_none() && embed.fields.is_empty() {
        respond_mistake!(&ctx, "You have to provide a description or embed fields!");
    }

    match webhook
        .execute(&ctx, false, ExecuteWebhook::new().embed(embed.into()))
        .await
    {
        Ok(_) => {
            ctx.say(format!("Successfully posted embed in {channel} channel."))
                .await?;
        }
        Err(e) => {
            respond_error!("Failed to post embed in {channel} channel", e, &ctx);
        }
    }

    Ok(())
}

/// Get the json of an embed to see what it should look like.
#[poise::command(slash_command, guild_only = true, ephemeral = true)]
pub async fn example(ctx: Context<'_>) -> anyhow::Result<()> {
    let attachment = CreateAttachment::path("src/assets/example_embed.json").await?;
    ctx.send(CreateReply::default().attachment(attachment))
        .await?;

    Ok(())
}
