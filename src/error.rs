use poise::FrameworkError;
use std::error::Error;

use crate::Context as AppContext;
use crate::Data;

#[allow(clippy::needless_lifetimes)]
pub async fn error_handler<'a>(
    error: FrameworkError<'a, Data, anyhow::Error>,
) -> anyhow::Result<()> {
    match error {
        FrameworkError::Command { error, ctx, .. } => {
            tracing::error!("Command error: {:?}", error);

            match ctx
                .reply("There was an error trying to execute that command.")
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    tracing::error!("Failed to send error message: {:?}", e);
                    Ok(())
                }
            }
        }
        FrameworkError::CommandPanic { payload, ctx, .. } => {
            tracing::error!("Command panic: {:?}", payload);

            match ctx
                .reply("Oops, something went terribly wrong. Please try again later.")
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    tracing::error!("Failed to send error message: {:?}", e);
                    Ok(())
                }
            }
        }
        FrameworkError::GuildOnly { ctx, .. } => {
            tracing::error!(
                "Guild-only command {} was used outside of a guild.",
                ctx.command().name.clone()
            );

            match ctx
                .reply("This command can only be used in a server.")
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    tracing::error!("Failed to send error message: {:?}", e);
                    Ok(())
                }
            }
        }
        FrameworkError::SubcommandRequired { ctx } => {
            tracing::error!(
                "Command {} requires a subcommand but none was provided.",
                ctx.command().name.clone()
            );

            match ctx.reply("This command requires a subcommand.").await {
                Ok(_) => Ok(()),
                Err(e) => {
                    tracing::error!("Failed to send error message: {:?}", e);
                    Ok(())
                }
            }
        }
        FrameworkError::EventHandler { error, event, .. } => {
            tracing::error!(
                "Event handler error for {}: {:#?}",
                event.snake_case_name(),
                error
            );

            Ok(())
        }
        FrameworkError::Setup {
            error,
            data_about_bot,
            ..
        } => {
            let username = data_about_bot.user.name.clone();
            tracing::error!("Failed to setup framework for {username}: {:#?}", error);

            Ok(())
        }
        other => {
            tracing::error!("Unhandled framework error: {:?}", other);

            Ok(())
        }
    }
}

#[macro_export]
macro_rules! respond_error {
    ($msg: literal, $err: expr, $ctx: expr) => {
        tracing::error!("{}: {:#?}", $msg, $err);
        $ctx.say(format!("{}.", $msg)).await?;
        return Ok(());
    };
}

#[macro_export]
macro_rules! respond_mistake {
    ($ctx: expr, $msg: literal) => {
        tracing::warn!(
            "{} invoked /{} but they made a mistake.",
            $ctx.author(),
            $ctx.command().name
        );
        $ctx.say($msg).await?;
        return Ok(());
    };
}
