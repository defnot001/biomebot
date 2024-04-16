use std::borrow::Cow;

use serenity::all::{GuildChannel, PartialGuild, User};

pub trait HasNameAndID {
    fn name(&self) -> &str;
    fn id(&self) -> Cow<str>;
}

pub enum TimestampStyle {
    ShortTime,
    LongTime,
    ShortDate,
    LongDate,
    ShortDateTime,
    LongDateTime,
    Relative,
}

impl HasNameAndID for User {
    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> Cow<str> {
        self.id.to_string().into()
    }
}

impl HasNameAndID for PartialGuild {
    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> Cow<str> {
        self.id.to_string().into()
    }
}

impl HasNameAndID for GuildChannel {
    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> Cow<str> {
        self.id.to_string().into()
    }
}

pub fn display(input: &impl HasNameAndID) -> String {
    format!("{} ({})", input.name(), input.id())
}

pub fn fdisplay(input: &impl HasNameAndID) -> String {
    format!(
        "{} ({})",
        escape_markdown(input.name()),
        inline_code(input.id())
    )
}

pub fn inline_code(input: impl Into<String>) -> String {
    format!("`{}`", input.into())
}

pub fn block_code(input: impl Into<String>) -> String {
    format!("```{}```", input.into())
}

pub fn user_mention(user: &User) -> String {
    format!("<@{}>", user.id)
}

pub fn escape_markdown(input: impl Into<String>) -> String {
    let input = input.into();
    let mut output = String::with_capacity(input.len());

    for c in input.chars() {
        if c.is_ascii_alphanumeric() {
            output.push(c)
        } else {
            output.extend(['\\', c])
        }
    }

    output
}

pub fn display_time(date_time: chrono::DateTime<chrono::Utc>) -> String {
    format!(
        "{}\n{}",
        time(date_time, TimestampStyle::LongDate),
        time(date_time, TimestampStyle::Relative)
    )
}

pub fn time(date_time: chrono::DateTime<chrono::Utc>, style: TimestampStyle) -> String {
    let timestamp = date_time.timestamp();

    match style {
        TimestampStyle::ShortTime => format!("<t:{timestamp}:t>"),
        TimestampStyle::LongTime => format!("<t:{timestamp}:T>"),
        TimestampStyle::ShortDate => format!("<t:{timestamp}:d>"),
        TimestampStyle::LongDate => format!("<t:{timestamp}:D>"),
        TimestampStyle::ShortDateTime => format!("<t:{timestamp}:f>"),
        TimestampStyle::LongDateTime => format!("<t:{timestamp}:F>"),
        TimestampStyle::Relative => format!("<t:{timestamp}:R>"),
    }
}

pub fn display_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 Bytes".to_string();
    }

    let sizes = ["Bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let i = (bytes as f64).log(1024.0).floor() as usize;
    let precise_count = bytes as f64 / 1024.0_f64.powi(i as i32);

    format!("{:.2} {}", precise_count, sizes[i])
}
