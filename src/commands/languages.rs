use std::{fmt::Display, str::FromStr};

use scraper::{selectable::Selectable, Html, Selector};

use crate::Context;

#[derive(Debug)]
struct LanguageFeature {
    language_name: String,
    parsing: LanguageSupportLevel,
    formatting: LanguageSupportLevel,
    linting: LanguageSupportLevel,
}

impl LanguageFeature {
    fn support_level_to_vec(&self) -> Vec<LanguageSupportLevel> {
        vec![self.parsing, self.formatting, self.linting]
    }

    pub fn is_fully_supported(&self) -> bool {
        self.support_level_to_vec()
            .iter()
            .all(|i| matches!(i, LanguageSupportLevel::Supported))
    }

    fn is_partially_supported(&self) -> bool {
        self.support_level_to_vec()
            .iter()
            .any(|i| matches!(i, LanguageSupportLevel::PartiallySupported))
    }

    fn is_work_in_progress(&self) -> bool {
        self.support_level_to_vec()
            .iter()
            .any(|i| matches!(i, LanguageSupportLevel::InProgress))
    }

    fn is_not_supported(&self) -> bool {
        self.support_level_to_vec()
            .iter()
            .all(|i| matches!(i, LanguageSupportLevel::NotInProgress))
    }
}

#[derive(Debug)]
struct SupportedLanguages {
    full: Vec<String>,
    partial: Vec<String>,
    wip: Vec<String>,
    nope: Vec<String>,
}

impl From<Vec<LanguageFeature>> for SupportedLanguages {
    fn from(languages: Vec<LanguageFeature>) -> Self {
        let mut full = Vec::new();
        let mut partial = Vec::new();
        let mut wip = Vec::new();
        let mut nope = Vec::new();

        for language in languages {
            if language.is_fully_supported() {
                full.push(language.language_name)
            } else if language.is_partially_supported() {
                partial.push(language.language_name)
            } else if language.is_work_in_progress() {
                wip.push(language.language_name)
            } else if language.is_not_supported() {
                nope.push(language.language_name)
            }
        }

        Self {
            full,
            partial,
            wip,
            nope,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(u8)]
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

    ctx.say(build_language_support_message(language_features))
        .await?;

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

fn build_language_support_message(language_features: Vec<LanguageFeature>) -> String {
    let titles = [
        "## :white_check_mark: Full Support",
        "## :warning: Partial Support",
        "## :hourglass_flowing_sand: Working on it",
        "## :no_entry: Not Yet Supported",
    ];

    let supported = SupportedLanguages::from(language_features);

    let languages = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
        titles[0],
        supported.full.join(", "),
        titles[1],
        supported.partial.join(", "),
        titles[2],
        supported.wip.join(", "),
        titles[3],
        supported.nope.join(", "),
    );

    format!("{languages}\nYou can find more details on [our website](https://biomejs.dev/internals/language-support/).")
}
