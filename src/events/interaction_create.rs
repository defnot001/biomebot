use poise::serenity_prelude as serenity;

use crate::util::format::display;

pub async fn handle_interaction_create(
    interaction: &serenity::Interaction,
    ctx: &serenity::Context,
) -> anyhow::Result<()> {
    if interaction.kind() != serenity::InteractionType::Command {
        return Ok(());
    }

    if let Some(command) = interaction.as_command() {
        match command.guild_id {
            Some(guild_id) => match guild_id.to_partial_guild(ctx).await {
                Ok(partial_guild) => {
                    let message = format!(
                        "{} used /{} in {}",
                        display(&command.user),
                        command.data.name,
                        display(&partial_guild)
                    );

                    tracing::info!(message);
                }
                Err(e) => {
                    tracing::error!("Error getting partial guild: {e}")
                }
            },
            None => {
                let message = format!(
                    "{} used /{} outside of a guild.",
                    display(&command.user),
                    command.data.name,
                );

                tracing::info!(message);
            }
        };
    }

    Ok(())
}
