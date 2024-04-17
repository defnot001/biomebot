use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, CreateEmbedFooter, User};

pub fn default_embed(user: &User) -> CreateEmbed {
    let footer = CreateEmbedFooter::new(format!(
        "Requested by {}",
        user.to_owned().global_name.unwrap_or(user.to_owned().name)
    ))
    .icon_url(user.static_avatar_url().unwrap_or_default());

    CreateEmbed::new()
        .color(6_530_042) // biome logo color
        .footer(footer)
        .timestamp(chrono::Utc::now())
}
