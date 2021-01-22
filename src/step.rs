use std::collections::HashMap;

use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use serde::Deserialize;

use crate::conventional_commits::update_project_from_conventional_commits;
use crate::state::State;
use crate::{command, git, issues, semver};

pub(crate) async fn run_step(step: Step, state: State) -> Result<State> {
    match step {
        Step::SelectJiraIssue { status } => issues::select_jira_issue(&status, state)
            .await
            .wrap_err("During SelectJiraIssue"),
        Step::SelectGitHubIssue { labels } => issues::select_github_issue(labels, state)
            .await
            .wrap_err("During SelectGitHubIssue"),
        Step::TransitionJiraIssue { status } => issues::transition_selected_issue(&status, state)
            .await
            .wrap_err("During TransitionJiraIssue"),
        Step::SwitchBranches => git::switch_branches(state).wrap_err("During SwitchBranches"),
        Step::RebaseBranch { to } => git::rebase_branch(state, &to).wrap_err("During MergeBranch"),
        Step::BumpVersion(rule) => {
            semver::bump_version(state, &rule).wrap_err("During BumpVersion")
        }
        Step::Command { command, variables } => {
            command::run_command(state, command, variables).wrap_err("During Command")
        }
        Step::UpdateProjectFromCommits => update_project_from_conventional_commits(state)
            .wrap_err("During UpdateProjectFromCommits"),
        Step::SelectIssueFromBranch => {
            git::select_issue_from_current_branch(state).wrap_err("During SelectIssueFromBranch")
        }
    }
}

/// Each variant describes an action you can take using Dobby, they are used when defining your
/// [`crate::Workflow`] via whatever config format is being utilized.
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Step {
    /// Search for Jira issues by status and display the list of them in the terminal.
    /// User is allowed to select one issue which will then change the workflow's state to
    /// [`State::IssueSelected`].
    ///
    /// ## Errors
    /// This step will fail if any of the following are true:
    /// 1. The workflow is already in [`State::IssueSelected`] before it executes.
    /// 2. Dobby cannot communicate with the configured Jira URL.
    /// 3. User does not select an issue.
    /// 4. There is no [`crate::config::Jira`] set.
    ///
    /// ## Example
    /// ```toml
    /// # dobby.toml
    /// [[workflows]]
    /// name = "Start some work"
    ///     [[workflows.steps]]
    ///     type = "SelectJiraIssue"
    ///     status = "Backlog"
    /// ```
    SelectJiraIssue {
        /// Issues with this status in Jira will be listed for the user to select.
        status: String,
    },
    /// Transition a Jira issue to a new status.
    ///
    /// ## Errors
    /// This step will fail when any of the following are true:
    /// 1. The workflow is not yet in [`State::IssueSelected`] ([`Step::SelectJiraIssue`] was not run
    ///     before this step).
    /// 2. Cannot communicate with Jira.
    /// 3. The configured status is invalid for the issue.
    /// 4. The selected issue is a GitHub issue instead of a Jira issue.
    ///
    /// ## Example
    /// ```toml
    /// # dobby.toml
    /// [[workflows]]
    /// name = "Start some work"
    ///     [[workflows.steps]]
    ///     type = "SelectJiraIssue"
    ///     status = "Backlog"
    ///     
    ///     [[workflows.steps]]
    ///     type = "TransitionJiraIssue"
    ///     status = "In Progress"
    /// ```
    TransitionJiraIssue {
        /// The status to transition the current issue to.
        status: String,
    },
    /// Search for GitHub issues by status and display the list of them in the terminal.
    /// User is allowed to select one issue which will then change the workflow's state to
    /// [`State::IssueSelected`].
    ///
    /// ## Errors
    /// This step will fail if any of the following are true:
    /// 1. The workflow is already in [`State::IssueSelected`] before it executes.
    /// 2. Dobby cannot communicate with GitHub.
    /// 4. There is no [`crate::config::GitHub`] set.
    /// 3. User does not select an issue.
    ///
    /// ## Example
    /// ```toml
    /// # dobby.toml
    /// [[workflows]]
    /// name = "Start some work"
    ///     [[workflows.steps]]
    ///     type = "SelectGitHubIssue"
    ///     label = "selected"
    /// ```
    SelectGitHubIssue {
        /// If provided, only issues with this label will be included
        labels: Option<Vec<String>>,
    },
    /// Attempt to parse issue info from the current branch name and change the workflow's state to
    /// [`State::IssueSelected`].
    ///
    /// ## Errors
    /// This step will fail if the current git branch cannot be determined or the name of that
    /// branch does not match the expected format. This is only intended to be used on branches
    /// which were created using the [`Step::SwitchBranches`] step.
    ///
    /// ## Example
    /// ```toml
    /// # dobby.toml
    /// [[workflows]]
    /// name = "Finish some work"
    ///     [[workflows.steps]]
    ///     type = "SelectIssueFromBranch"
    ///     
    ///     [[workflows.steps]]
    ///     type = "TransitionJiraIssue"
    ///     status = "QA"
    /// ```
    SelectIssueFromBranch,
    /// Uses the name of the currently selected issue to checkout an existing or create a new
    /// branch for development. If an existing branch is not found, the user will be prompted to
    /// select an existing local branch to base the new branch off of. Remote branches are not
    /// shown.
    ///
    /// ## Errors
    /// This step fails if any of the following are true.
    /// 1. Workflow is not in [`State::IssueSelected`], as [`Step::SelectJiraIssue`] or [`Step::SelectGitHubIssue`]
    ///     were not run before this step.
    /// 2. Current directory is not a Git repository
    ///
    /// ## Example
    /// ```toml
    /// # dobby.toml
    /// [[workflows]]
    /// name = "Start some work"
    ///     [[workflows.steps]]
    ///     type = "SelectIssue"
    ///     status = "Backlog"
    ///     
    ///     [[workflows.steps]]
    ///     type = "SwitchBranches"
    /// ```
    SwitchBranches,
    /// Rebase the current branch onto the branch defined by `to`.
    ///
    /// ## Errors
    /// Fails if any of the following are true:
    /// 1. The current directory is not a Git repository.
    /// 2. The `to` branch cannot be found locally (does not check remotes).
    /// 3. The repo is not on the tip of a branch (e.g. detached HEAD)
    /// 4. Rebase fails (e.g. not a clean working tree)
    ///
    /// ## Example
    /// ```toml
    /// # dobby.toml
    /// [[workflows]]
    /// name = "Finish some work"
    ///     [[workflows.steps]]
    ///     type = "RebaseBranch"
    ///     to = "main"
    /// ```
    RebaseBranch {
        /// The branch to rebase onto.
        to: String,
    },
    /// Bump the version of the project in any supported formats found using a
    /// [Semantic Versioning](https://semver.org) rule.
    ///
    /// ## Supported Formats
    /// These are the types of files that this step knows how to search for a semantic version and
    /// bump:
    /// 1. Cargo.toml in the current directory.
    ///
    /// ## Rules
    /// Details about the rules that can be provided to this step can be found in [`semver::Rule`].
    ///
    /// ## Errors
    /// This step will fail if any of the following are true:
    /// 1. A malformed version string is found while attempting to bump.
    ///
    /// ## Example
    /// ```toml
    /// [[workflows.steps]]
    /// type = "BumpVersion"
    /// rule = "Pre"
    /// value = "rc"
    /// ```
    BumpVersion(crate::semver::Rule),
    /// Run a command in your current shell after optionally replacing some variables.
    ///
    /// ## Example
    /// If the current version for your project is "1.0.0", the following workflow step will run
    /// `git tag v.1.0.0` in your current shell.
    ///
    /// ```toml
    /// [[workflows.steps]]
    /// type = "Command"
    /// command = "git tag v.version"
    /// variables = {"version" = "Version"}
    /// ```
    ///
    /// Note that the key ("version" in the example) is completely up to you, make it whatever you
    /// like, but if it's not found in the command string it won't be substituted correctly.
    Command {
        /// The command to run, with any variable keys you wish to replace.
        command: String,
        /// A map of value-to-replace to [Variable][`crate::command::Variable`] to replace
        /// it with.
        variables: Option<HashMap<String, command::Variable>>,
    },
    /// This will look through all commits since the last tag and parse any
    /// [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) it finds. It will
    /// then bump the project version (depending on the rule determined from the commits) and add
    /// a new Changelog entry using the [Keep A Changelog](https://keepachangelog.com/en/1.0.0/)
    /// format.
    ///
    /// ## Limitations
    /// The CHANGELOG format is pretty strict, it needs to have at least one version already in it
    /// and every version needs to be a level 2 header (`## 1.0.0`). Only three sections will be
    /// added to the new version, `### Breaking Changes` for anything that conventional commits have
    /// marked as breaking, `### Fixes` for anything called `fix:`, and `### Features` for anything
    /// with `feat: `. Any other commits (conventional or not) will be left out. A new version will
    /// __always__ be generated though, even if there are no changes to record.
    ///
    /// ## Errors
    /// The reasons this can fail:
    /// 1. If there is no previous tag to base changes off of.
    /// 1. There is no CHANGELOG.md file in the working directory to update.
    /// 1. The version could not be bumped for some reason.
    UpdateProjectFromCommits,
}
