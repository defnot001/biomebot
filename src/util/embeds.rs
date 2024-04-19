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

#[derive(Default, poise::ChoiceParameter)]
pub enum EmbedColor {
    #[default]
    Biome = 0x63A3FA,
    Black = 0x000000,
    Gray = 0xBEBEBE,
    White = 0xFFFFFF,
    Blue = 0x0000FF,
    Cyan = 0x00FFFF,
    Green = 0x00FF00,
    Orange = 0xFFA500,
    Coral = 0xFF7F50,
    Red = 0xFF0000,
    DeepPink = 0xFF1493,
    Purple = 0xA020F0,
    Magenta = 0xFF00FF,
    Yellow = 0xFFFF00,
    Gold = 0xFFD700,
    None = 0x2F3136,
}

impl From<EmbedColor> for serenity::Colour {
    fn from(colour: EmbedColor) -> Self {
        serenity::Colour(colour as u32)
    }
}
