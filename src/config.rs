use poise::serenity_prelude as serenity;
use serde::Deserialize;
use serenity::GuildId;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub bot: BotConfig,
    pub github: GithubConfig,
    pub webserver: WebserverConfig,
    pub database: DatabaseConfig,
}

impl Config {
    pub fn load() -> Self {
        let config_file = std::fs::File::open("config.json").expect("Failed to open config.json");
        let reader = std::io::BufReader::new(config_file);

        serde_json::from_reader(reader).expect("Failed to parse config.json")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    pub token: String,
    pub guild_id: GuildId,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GithubConfig {
    pub webhook_secret: String,
    pub target_webhook: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebserverConfig {
    pub host: [u8; 4],
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}
