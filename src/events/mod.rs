pub mod interaction_create;
pub mod ready;

use poise::serenity_prelude as serenity;

use crate::Data;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, anyhow::Error>,
) -> anyhow::Result<()> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            ready::handle_ready(data_about_bot, ctx).await?;
        }
        serenity::FullEvent::InteractionCreate { interaction, .. } => {
            interaction_create::handle_interaction_create(interaction, ctx).await?;
        }
        _ => {}
    }
    Ok(())
}
