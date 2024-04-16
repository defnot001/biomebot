use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
};

use crate::config::Config;

pub async fn handle_gh(state: State<Config>, req: Request) -> StatusCode {
    let Some(headers) = extract_headers(req.headers()) else {
        return StatusCode::UNAUTHORIZED;
    };

    println!("{:#?}", headers);

    StatusCode::OK
}

fn extract_headers(headers: &HeaderMap) -> Option<GithubHeaders> {
    let content_type = headers.get("content-type")?.to_str().ok()?.to_string();
    let github_event = headers.get("x-github-event")?.to_str().ok()?.to_string();
    let signature_sha256 = headers
        .get("x-hub-signature-256")?
        .to_str()
        .ok()?
        .to_string();

    Some(GithubHeaders {
        content_type,
        github_event,
        signature_sha256,
    })
}

#[derive(Debug)]
struct GithubHeaders {
    content_type: String,
    github_event: String,
    signature_sha256: String,
}

#[allow(unused)]
enum GithubEvent {
    /// This event occurs when there is a change to branch protection configurations for a repository.
    BranchProtectionConfiguration,
    /// This event occurs when there is activity relating to branch protection rules.
    BranchProtectionRule,
    /// This event occurs when there is activity relating to a check run.
    CheckRun,
    /// This event occurs when there is activity relating to a check suite.
    CheckSuite,
    /// This event occurs when there is activity relating to code scanning alerts in a repository.
    CodeScanningAlert,
    /// This event occurs when there is activity relating to commit comments.
    CommitComment,
    /// This event occurs when a Git branch or tag is created.
    Create,
    /// This event occurs when there is activity relating to a custom property.
    CustomProperty,
    /// This event occurs when there is activity relating to custom property values for a repository.
    CustomPropertyValues,
    /// This event occurs when a Git branch or tag is deleted.
    Delete,
    /// This event occurs when there is activity relating to Dependabot alerts.
    DependabotAlert,
    /// This event occurs when there is activity relating to deploy keys.
    DeployKey,
    /// This event occurs when there is activity relating to deployments.
    Deployment,
    /// This event occurs when there is activity relating to deployment protection rules.
    DeploymentProtectionRule,
    /// This event occurs when there is activity relating to deployment reviews.
    DeploymentReview,
    /// This event occurs when there is activity relating to deployment statuses.
    DeploymentStatus,
    /// This event occurs when there is activity relating to a discussion.
    /// Note: Webhook events for GitHub Discussions are currently in beta and subject to change.
    Discussion,
    /// This event occurs when there is activity relating to a comment on a discussion.
    /// Note: Webhook events for GitHub Discussions are currently in beta and subject to change.
    DiscussionComment,
    /// This event occurs when someone forks a repository.
    Fork,
    /// This event occurs when a user revokes their authorization of a GitHub App.
    GithubAppAuthorization,
    /// This event occurs when someone creates or updates a wiki page.
    Gollum,
    /// This event occurs when there is activity relating to a GitHub App installation.
    Installation,
    /// This event occurs when there is activity relating to which repositories a GitHub App installation can access.
    InstallationRepositories,
    /// This event occurs when there is activity relating to a comment on an issue or pull request.
    IssueComment,
    /// This event occurs when there is activity relating to an issue.
    Issues,
    /// This event occurs when there is activity relating to labels.
    Label,
    /// This event occurs when there is activity relating to a GitHub Marketplace purchase.
    MarketplacePurchase,
    /// This event occurs when there is activity relating to collaborators in a repository.
    Member,
    /// This event occurs when there is activity relating to team membership.
    Membership,
    /// This event occurs when there is activity relating to a merge group in a merge queue.
    MergeGroup,
    // This event occurs when there is activity relating to a webhook itself.
    Meta,
    /// This event occurs when there is activity relating to milestones.
    Milestone,
    /// This event occurs when organization owners or moderators block or unblock a non-member from collaborating on the organization's repositories.
    OrgBlock,
    /// This event occurs when there is activity relating to an organization and its members.
    Organization,
    /// This event occurs when there is activity relating to GitHub Packages.
    Package,
    /// This event occurs when there is an attempted build of a GitHub Pages site.
    PageBuild,
    /// This event occurs when there is activity relating to a request for a fine-grained personal access token to access resources that belong to a resource owner that requires approval for token access.
    PersonalAccessTokenRequest,
    /// This event occurs when you create a new webhook. The ping event is a confirmation from GitHub that you configured the webhook correctly.
    Ping,
    /// This event occurs when there is activity relating to a card on a project (classic).
    ProjectCard,
    /// This event occurs when there is activity relating to a project (classic).
    Project,
    /// This event occurs when there is activity relating to a column on a project (classic).
    ProjectColumn,
    /// This event occurs when there is activity relating to an organization-level project.
    ProjectsV2,
    /// This event occurs when there is activity relating to an item on an organization-level project.
    ProjectsV2Item,
    /// This event occurs when repository visibility changes from private to public.
    Public,
    /// This event occurs when there is activity on a pull request.
    PullRequest,
    /// This event occurs when there is activity relating to a pull request review comment.
    PullRequestReviewComment,
    /// This event occurs when there is activity relating to a pull request review.
    PullRequestReview,
    /// This event occurs when there is activity relating to a comment thread on a pull request.
    PullRequestReviewThread,
    /// This event occurs when there is a push to a repository branch.
    Push,
    /// This event occurs when there is activity relating to GitHub Packages.
    RegistryPackage,
    /// This event occurs when there is activity relating to releases.
    Release,
    /// This event occurs when there is activity relating to a repository security advisory.
    RepositoryAdvisory,
    /// This event occurs when there is activity relating to repositories.
    Repository,
    /// This event occurs when a GitHub App sends a POST request to /repos/{owner}/{repo}/dispatches
    RepositoryDispatch,
    /// This event occurs when a repository is imported to GitHub.
    RepositoryImport,
    /// This event occurs when there is activity relating to repository rulesets.
    RepositoryRuleset,
    /// This event occurs when there is activity relating to a security vulnerability alert in a repository.
    /// Note: This event is deprecated. Use the dependabot_alert event instead.
    RepositoryVulnerabilityAlert,
    /// This event occurs when there is activity relating to a secret scanning alert.
    SecretScanningAlert,
    /// This event occurs when there is activity relating to the locations of a secret in a secret scanning alert.
    SecretScanningAlertLocation,
    /// This event occurs when there is activity relating to a global security advisory that was reviewed by GitHub.
    /// A GitHub-reviewed global security advisory provides information about security vulnerabilities or malware that have been mapped to packages in ecosystems we support.
    SecurityAdvisory,
    /// This event occurs when code security and analysis features are enabled or disabled for a repository.
    SecurityAndAnalysis,
    /// This event occurs when there is activity relating to a sponsorship listing.
    Sponsorship,
    /// This event occurs when there is activity relating to repository stars.
    Star,
    /// This event occurs when the status of a Git commit changes.
    /// For example, commits can be marked as `error`, `failure`, `pending`, or `success`.
    Status,
    /// This event occurs when a team is added to a repository.
    TeamAdd,
    /// This event occurs when there is activity relating to teams in an organization.
    Team,
    /// This event occurs when there is activity relating to watching, or subscribing to, a repository.
    Watch,
    /// This event occurs when a GitHub Actions workflow is manually triggered.
    WorkflowDispatch,
    /// This event occurs when there is activity relating to a job in a GitHub Actions workflow.
    WorkflowJob,
    /// This event occurs when there is activity relating to a run of a GitHub Actions workflow.
    WorkflowRun,
}
