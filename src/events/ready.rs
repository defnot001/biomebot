use poise::serenity_prelude as serenity;

pub async fn handle_ready(
    data_about_bot: &serenity::Ready,
    ctx: &serenity::Context,
) -> anyhow::Result<()> {
    tracing::info!("Logged in as {}.", data_about_bot.user.name);

    ctx.set_activity(Some(serenity::ActivityData {
        name: "commands".to_string(),
        kind: serenity::ActivityType::Listening,
        state: None,
        url: None,
    }));

    Ok(())
}
