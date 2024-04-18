#![allow(unused, dead_code)]

mod commands;
mod config;
mod error;
mod events;
mod routes;
mod util;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{routing::post, Router};
use commands::languages;
use config::Config;
use events::event_handler;
use poise::serenity_prelude as serenity;
use sqlx::postgres::PgPoolOptions;

use crate::routes::github::handle_gh;

#[derive(Debug, Clone)]
pub struct Data {
    db_pool: sqlx::PgPool,
    config: Config,
}

pub type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::fmt().compact().finish())?;
    tracing::info!("Logger initialized.");

    let config = Config::load();
    tracing::info!("Config loaded.");

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database.url)
        .await?;
    tracing::info!("Database connected.");

    let data = Data { config, db_pool };

    let discord_handle = tokio::spawn(setup_bot(data.clone()));
    let webserver_handle = tokio::spawn(setup_webserver(data));

    let (discord_result, webserver_result) = tokio::join!(discord_handle, webserver_handle);

    discord_result??;
    webserver_result??;

    Ok(())
}

async fn setup_bot(data: Data) -> anyhow::Result<()> {
    let client_intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let register_guild_id = data.config.bot.guild_id;
    let bot_token = data.config.bot.token.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![languages::languages()],
            event_handler: |ctx, event, framework, _data| {
                Box::pin(event_handler(ctx, event, framework))
            },
            on_error: |error| {
                Box::pin(async move {
                    error::error_handler(error)
                        .await
                        .expect("Failed to recover from error!");
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    register_guild_id,
                )
                .await?;
                Ok(data)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(bot_token, client_intents)
        .framework(framework)
        .await;

    client?.start().await?;

    Ok(())
}

async fn setup_webserver(data: Data) -> anyhow::Result<()> {
    let web_app = Router::new()
        .route("/github", post(handle_gh))
        .fallback(routes::not_found::handle_404)
        .with_state(data.clone());

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::from(data.config.webserver.host)),
        data.config.webserver.port,
    ))
    .await?;

    tracing::info!(
        "Webserver listening on port {}.",
        data.config.webserver.port
    );

    axum::serve(
        listener,
        web_app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
