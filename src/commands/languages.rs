use std::{fmt::Display, str::FromStr};

use poise::serenity_prelude as serenity;
use poise::CreateReply;
use scraper::{selectable::Selectable, Html, Selector};
use serenity::{CreateEmbed, User};

use crate::{util::embeds::default_embed, Context};

#[derive(Debug)]
struct LanguageFeature {
    language_name: String,
    parsing: LanguageSupportLevel,
    formatting: LanguageSupportLevel,
    linting: LanguageSupportLevel,
}

impl LanguageFeature {
    fn to_embed_field(&self) -> (String, String, bool) {
        let value = format!(
            "Parsing {} \u{00A0}\u{00A0} Formatting {} \u{00A0}\u{00A0} Linting {}",
            self.parsing, self.formatting, self.linting
        );

        (self.language_name.clone(), value, false)
    }
}

#[derive(Debug)]
enum LanguageSupportLevel {
    Supported,
    InProgress,
    PartiallySupported,
    NotInProgress,
}

impl Display for LanguageSupportLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Supported => write!(f, "\u{2705}"),
            Self::InProgress => write!(f, "\u{231B}\u{FE0F}"),
            Self::PartiallySupported => write!(f, "\u{26A0}\u{FE0F}"),
            Self::NotInProgress => write!(f, "\u{1F6AB}"),
        }
    }
}

impl FromStr for LanguageSupportLevel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        match s.trim() {
            "\u{2705}\u{FE0F}" | "\u{2705}" => Ok(Self::Supported),
            "\u{231B}\u{FE0F}" => Ok(Self::InProgress),
            "\u{26A0}\u{FE0F}" => Ok(Self::PartiallySupported),
            "\u{1F6AB}" => Ok(Self::NotInProgress),
            _ => {
                anyhow::bail!("Unsupported Language support level: {}", s,);
            }
        }
    }
}

/// See the status of Biome's supported languages.
#[poise::command(slash_command, guild_only = true)]
pub async fn languages(ctx: Context<'_>) -> anyhow::Result<()> {
    ctx.defer().await?;

    let language_features = scrape_language_support().await?;
    let embed = build_language_support_embed(ctx.author(), language_features);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

async fn scrape_language_support() -> anyhow::Result<Vec<LanguageFeature>> {
    let response = reqwest::get("https://biomejs.dev/internals/language-support")
        .await?
        .text()
        .await?;

    let document = Html::parse_document(&response);
    let table_selector = parse_selector("table")?;

    let Some(first_table) = document.select(&table_selector).next() else {
        anyhow::bail!("Failed to find first table in the document")
    };

    let tr_selector = parse_selector("tr")?;
    let td_selector = parse_selector("td")?;

    let mut features = Vec::new();

    for (i, element) in first_table.select(&tr_selector).enumerate() {
        if i == 0 {
            continue;
        }; // ain't nobody wanna look at that header.

        let columns = element.select(&td_selector).collect::<Vec<_>>();

        if columns.len() != 4 {
            anyhow::bail!("Encountered unexpexted HTML in the supported languages table");
        }

        let language_name = columns[0].text().collect::<String>();
        let parsing = LanguageSupportLevel::from_str(columns[1].text().collect::<String>().trim())?;
        let formatting =
            LanguageSupportLevel::from_str(columns[2].text().collect::<String>().trim())?;
        let linting = LanguageSupportLevel::from_str(columns[3].text().collect::<String>().trim())?;

        features.push(LanguageFeature {
            language_name,
            parsing,
            formatting,
            linting,
        })
    }

    Ok(features)
}

fn parse_selector(sel: &str) -> anyhow::Result<Selector> {
    match Selector::parse(sel) {
        Ok(s) => Ok(s),
        Err(e) => {
            anyhow::bail!("Error parsing selector: {e}");
        }
    }
}

fn build_language_support_embed(
    interaction_user: &User,
    language_features: Vec<LanguageFeature>,
) -> CreateEmbed {
    let website_link = "You can find more details on [our website](https://biomejs.dev/internals/language-support/).";
    let legend = "\u{2705} Supported\n\u{1F6AB} Not in progress\n\u{231B}\u{FE0F} In progress\n\u{26A0}\u{FE0F}: Partially supported";
    let fields = language_features
        .into_iter()
        .map(|f| f.to_embed_field())
        .collect::<Vec<_>>();

    let description = format!("{website_link}\n\n{legend}");

    default_embed(interaction_user)
        .title("Biome's Currently Supported Languages")
        .description(description)
        .fields(fields)
}
