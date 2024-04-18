use std::{fmt::Display, str::FromStr};

use anyhow::Context;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::Value;
use serenity::all::{
    CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, ExecuteWebhook, Http, Webhook,
};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::Data;

#[derive(Debug)]
enum GithubEvent {
    Issues,
    PullRequest,
}

impl Display for GithubEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Issues => write!(f, "issues"),
            Self::PullRequest => write!(f, "pull_request"),
        }
    }
}

impl FromStr for GithubEvent {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "issues" => Ok(Self::Issues),
            "pull_request" => Ok(Self::PullRequest),
            _ => {
                anyhow::bail!("Received unrecognized event: {s}");
            }
        }
    }
}

#[derive(Debug)]
enum GithubIssuesAction {
    /// An issue was assigned to a user.
    Assigned,
    /// An issue was closed.
    Closed,
    /// An issue was deleted.
    Deleted,
    /// An issue was removed from a milestone.
    Demilestoned,
    /// The title or body on an issue was edited.
    Edited,
    /// A label was added to an issue.
    Labeled,
    /// Conversation on an issue was locked.
    Locked,
    /// An issue was added to a milestone.
    Milestoned,
    /// An issue was created. When a closed issue is reopened, the action will be `reopened` instead.
    Opened,
    /// An issue was pinned to a repository.
    Pinned,
    /// A closed issue was reopened.
    Reopened,
    /// An issue was transferred to another repository.
    Transferred,
    /// A user was unassigned from an issue.
    Unassigned,
    /// A label was removed from an issue.
    Unlabeled,
    /// Conversation on an issue was locked. The official github docs are wrong on this one.
    Unlocked,
    /// An issue was unpinned from a repository.
    Unpinned,
}

impl Display for GithubIssuesAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assigned => write!(f, "assigned"),
            Self::Closed => write!(f, "closed"),
            Self::Deleted => write!(f, "deleted"),
            Self::Demilestoned => write!(f, "demilestoned"),
            Self::Edited => write!(f, "edited"),
            Self::Labeled => write!(f, "labeled"),
            Self::Locked => write!(f, "locked"),
            Self::Milestoned => write!(f, "milestoned"),
            Self::Opened => write!(f, "opened"),
            Self::Pinned => write!(f, "pinned"),
            Self::Reopened => write!(f, "reopened"),
            Self::Transferred => write!(f, "transferred"),
            Self::Unassigned => write!(f, "unassigned"),
            Self::Unlabeled => write!(f, "unlabeled"),
            Self::Unlocked => write!(f, "unlocked"),
            Self::Unpinned => write!(f, "unpinned"),
        }
    }
}

impl FromStr for GithubIssuesAction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "assigned" => Ok(Self::Assigned),
            "closed" => Ok(Self::Closed),
            "deleted" => Ok(Self::Deleted),
            "demilestoned" => Ok(Self::Demilestoned),
            "edited" => Ok(Self::Edited),
            "labeled" => Ok(Self::Labeled),
            "locked" => Ok(Self::Locked),
            "milestoned" => Ok(Self::Milestoned),
            "opened" => Ok(Self::Opened),
            "pinned" => Ok(Self::Pinned),
            "reopened" => Ok(Self::Reopened),
            "transferred" => Ok(Self::Transferred),
            "unassigned" => Ok(Self::Unassigned),
            "unlabeled" => Ok(Self::Unlabeled),
            "unlocked" => Ok(Self::Unlocked),
            "unpinned" => Ok(Self::Unpinned),
            _ => {
                anyhow::bail!("Unknown Github Issue Action: {s}");
            }
        }
    }
}

impl GithubIssuesAction {
    fn is_label(&self) -> bool {
        matches!(self, Self::Labeled | Self::Unlabeled)
    }
}

#[derive(Debug, Deserialize)]
struct GithubIssueLabelEvent {
    action: String,
    issue: GithubIssue,
    label: Option<GithubIssueLabel>,
    repository: GithubRepository,
    sender: GithubUser,
}

impl GithubIssueLabelEvent {
    /// This function returns true when multiple conditions are met at the same time:
    ///
    /// The issue has to be open.
    /// The label `good-first-issue` was added.
    fn should_report(&self) -> bool {
        self.action == "labeled"
            && self.issue.state == "open"
            && self
                .label
                .as_ref()
                .map_or(false, |label| label.name == "good first issue")
    }
}

#[derive(Debug, Deserialize)]
struct GithubIssue {
    /// can be one of `resolved`, `off-topic`, `too heated`, `spam` or `None`
    active_lock_reason: Option<String>,
    assignees: Vec<Option<GithubUser>>,
    author_association: String,
    body: Option<String>,
    labels: Vec<GithubIssueLabel>,
    node_id: String,
    number: i64,
    repository_url: String,
    /// State of the issue; either 'open' or 'closed'
    state: String,
    title: String,
    url: String,
    html_url: String,
    user: Option<GithubUser>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct GithubUser {
    id: u64,
    login: String,
    #[serde(rename = "type")]
    /// Can be one of: `Bot`, `User`, `Organization`, `Mannequin`
    user_type: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubIssueLabel {
    /// 6-character hex code, without the leading #, identifying the color
    color: String,
    default: bool,
    description: Option<String>,
    id: u64,
    /// The name of the label.
    name: String,
    node_id: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct GithubRepository {
    id: i64,
    node_id: String,
    name: String,
    full_name: String,
    private: bool,
}

type HmacSha256 = Hmac<Sha256>;

pub async fn handle_gh(State(data): State<Data>, headers: HeaderMap, body: Bytes) -> StatusCode {
    tracing::info!("Received POST request at /github.");

    let body_bytes = body.as_ref();

    if !is_authorized(&headers, body_bytes, &data.config.github.webhook_secret) {
        tracing::warn!("Unauthorized request at /github!");
        return StatusCode::UNAUTHORIZED;
    }

    if is_issues_event(&headers) {
        match handle_issues(body_bytes, data).await {
            Ok(_) => return StatusCode::OK,
            Err(e) => {
                tracing::error!("Error processing github event: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    }

    let json: Value = match serde_json::from_slice(body_bytes) {
        Ok(json) => json,
        Err(_) => {
            tracing::warn!("Wrong formatted request at /github!");
            return StatusCode::BAD_REQUEST;
        }
    };

    if !is_human_user(&json) {
        return StatusCode::OK;
    }

    match post_to_activity_webhook(data.config.github.activity_webhook, body, headers).await {
        Ok(_) => {
            tracing::info!("Forwarded github event to webhook.");
            StatusCode::OK
        }
        Err(e) => {
            tracing::info!("Failed to forward github event: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

fn is_issues_event(headers: &HeaderMap) -> bool {
    let Some(event_header) = headers.get("x-github-event").and_then(|h| h.to_str().ok()) else {
        return false;
    };

    matches!(GithubEvent::from_str(event_header), Ok(GithubEvent::Issues))
}

fn is_authorized(headers: &HeaderMap, body: &[u8], secret: &str) -> bool {
    let header_signature = match extract_signature(headers) {
        Some(s) => s,
        None => return false,
    };

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    mac.update(body);
    let calculated_signature = mac.finalize().into_bytes();

    let header_signature = match hex::decode(header_signature) {
        Ok(s) => s,
        Err(_) => return false,
    };

    header_signature.ct_eq(&calculated_signature).into()
}

fn extract_signature(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-hub-signature-256")
        .and_then(|hv| hv.to_str().ok())
        .map(|s| s.trim_start_matches("sha256=").to_string())
}

fn is_human_user(json: &Value) -> bool {
    json.get("sender")
        .and_then(|sender| sender.get("type"))
        .and_then(|user_type| user_type.as_str())
        .map_or(false, |user_type| user_type == "User")
}

async fn post_to_activity_webhook(
    activity_webhook: String,
    body: Bytes,
    headers: HeaderMap,
) -> anyhow::Result<()> {
    let mut forward_headers = HeaderMap::new();

    for (key, value) in headers.iter() {
        if key != "authorization" && key != "host" {
            forward_headers.insert(key.clone(), value.clone());
        }
    }

    let res = reqwest::Client::new()
        .post(activity_webhook)
        .headers(forward_headers)
        .body(body)
        .send()
        .await?;

    if !res.status().is_success() {
        anyhow::bail!("Failed to send message to webhook: {}", res.status())
    }

    Ok(())
}

async fn handle_issues(body: &[u8], data: Data) -> anyhow::Result<()> {
    if !get_issue_action(body)?.is_label() {
        return Ok(());
    }

    let label_event: GithubIssueLabelEvent = serde_json::from_slice(body)?;

    if label_event.should_report() {
        post_good_first_issue(
            label_event,
            &data.config.github.issues_webhook,
            &data.config.bot.token,
        )
        .await?
    }

    Ok(())
}

fn get_issue_action(body: &[u8]) -> anyhow::Result<GithubIssuesAction> {
    GithubIssuesAction::from_str(
        serde_json::from_slice::<Value>(body)?
            .get("action")
            .context("Json body for issue event is missing required `action` field")?
            .as_str()
            .context("Field `action` on issues json body is not a string.")?,
    )
}

async fn post_good_first_issue(
    label_event: GithubIssueLabelEvent,
    issues_webhook_url: &str,
    bot_token: &str,
) -> anyhow::Result<()> {
    let http = Http::new(bot_token);
    let webhook = Webhook::from_url(&http, issues_webhook_url).await?;

    let description = format!("**{}** just added label `good-first-issue` to [issue #{}]({}) ({}) in the {} repository. This is a good chance to get your first contribution!",
        label_event.sender.login,
        label_event.issue.number,
        label_event.issue.html_url,
        label_event.issue.title,
        label_event.repository.name
    );

    let embed_author = if let Some(avatar_url) = label_event.sender.avatar_url {
        CreateEmbedAuthor::new(label_event.sender.login).icon_url(avatar_url)
    } else {
        CreateEmbedAuthor::new(label_event.sender.login)
    };

    let embed = CreateEmbed::new()
        .color(6_530_042) // biome logo color
        .author(embed_author)
        .title("New good first issue alert")
        .description(description)
        .footer(CreateEmbedFooter::new("Biome Issue Tracker"))
        .timestamp(chrono::Utc::now());

    webhook
        .execute(&http, false, ExecuteWebhook::default().embed(embed))
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash() {
        let secret = "It's a Secret to Everybody";
        let payload = "Hello, World!";

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let result = hex::encode(mac.finalize().into_bytes());

        let expected = "757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17";

        assert_eq!(result, expected);
    }
}
