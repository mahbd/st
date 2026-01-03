//! `submit` subcommand.

use crate::{
    ctx::StContext,
    errors::{StError, StResult},
    git::RepositoryExt,
    tree::RemoteMetadata,
};
use clap::Args;
use git2::BranchType;
use nu_ansi_term::Color;
use octocrab::{issues::IssueHandler, models::CommentId, pulls::PullRequestHandler, Octocrab};

/// CLI arguments for the `submit` subcommand.
#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct SubmitCmd {
    /// Force the submission of the stack, analogous to `git push --force`.
    #[clap(long, short)]
    force: bool,
    /// Submit all tracked branches, not just the current stack.
    #[clap(long, short)]
    all: bool,
}

impl SubmitCmd {
    /// Run the `submit` subcommand.
    pub async fn run(self, mut ctx: StContext<'_>) -> StResult<()> {
        // Establish the GitHub API client.
        let gh_client = Octocrab::builder()
            .personal_token(ctx.cfg.github_token.clone())
            .build()?;
        let (owner, repo) = ctx.owner_and_repository()?;
        let mut pulls = gh_client.pulls(&owner, &repo);

        // Resolve the branches to submit
        let branches_to_submit = if self.all {
            // Submit all tracked branches
            ctx.tree.branches()?
        } else {
            // Submit only the current stack
            ctx.discover_stack()?
        };

        // Perform pre-flight checks.
        println!("🔍 Checking for closed pull requests...");
        self.pre_flight(&mut ctx, &branches_to_submit, &mut pulls).await?;

        // Submit the stack.
        println!(
            "\n🐙 Submitting changes to remote `{}`...",
            Color::Blue.paint("origin")
        );
        self.submit_stack(&mut ctx, &mut pulls, &owner, &repo)
            .await?;

        // Update the stack navigation comments on the PRs.
        println!("\n📝 Updating stack navigation comments...");
        self.update_pr_comments(&mut ctx, gh_client.issues(owner, repo), &branches_to_submit)
            .await?;

        println!("\n🧙💫 All pull requests up to date.");
        Ok(())
    }

    /// Performs pre-flight checks before submitting the stack.
    async fn pre_flight(
        &self,
        ctx: &mut StContext<'_>,
        stack: &[String],
        pulls: &mut PullRequestHandler<'_>,
    ) -> StResult<()> {
        // Return early if the stack is not restacked or the current working tree is dirty.
        ctx.check_cleanliness(stack)?;

        // Check if any PRs have been closed, and offer to delete them before starting the submission process.
        let num_closed = ctx
            .delete_closed_branches(
                stack.iter().skip(1).cloned().collect::<Vec<_>>().as_slice(),
                pulls,
            )
            .await?;

        if num_closed > 0 {
            println!(
                "Deleted {} closed pull request{}. Run `{}` to re-stack the branches.",
                Color::Red.paint(num_closed.to_string()),
                (num_closed != 1).then_some("s").unwrap_or_default(),
                Color::Blue.paint("st restack")
            );
        }

        Ok(())
    }

    /// Submits the stack of branches to GitHub.
    async fn submit_stack(
        &self,
        ctx: &mut StContext<'_>,
        pulls: &mut PullRequestHandler<'_>,
        owner: &str,
        repo: &str,
    ) -> StResult<()> {
        // Get all branches to process
        let all_branches = if self.all {
            ctx.tree.branches()?
        } else {
            ctx.discover_stack()?
        };

        // Iterate over the branches and submit PRs.
        for branch in all_branches.iter().skip(1) {
            // Get the parent for this specific branch
            let parent = {
                let tracked_branch = ctx
                    .tree
                    .get(branch)
                    .ok_or_else(|| StError::BranchNotTracked(branch.to_string()))?;
                
                tracked_branch
                    .parent
                    .clone()
                    .ok_or_else(|| StError::BranchNotTracked(format!("Parent not found for {}", branch)))?
            };

            let tracked_branch = ctx
                .tree
                .get_mut(branch)
                .ok_or_else(|| StError::BranchNotTracked(branch.to_string()))?;

            if let Some(remote_meta) = tracked_branch.remote.as_ref() {
                // If the PR has already been submitted.

                // Grab remote metadata for the pull request.
                let remote_pr = pulls.get(remote_meta.pr_number).await?;

                // Check if the PR base needs to be updated
                if &remote_pr.base.ref_field != &parent {
                    // Update the PR base.
                    pulls
                        .update(remote_meta.pr_number)
                        .base(&parent)
                        .send()
                        .await?;
                    println!(
                        "-> Updated base branch for pull request for branch `{}` to `{}`.",
                        Color::Green.paint(branch),
                        Color::Yellow.paint(&parent)
                    );
                }

                // Check if the local branch is ahead of the remote.
                let remote_synced = remote_pr.head.sha
                    == ctx
                        .repository
                        .find_branch(branch, BranchType::Local)?
                        .get()
                        .target()
                        .ok_or(StError::BranchUnavailable)?
                        .to_string();
                if remote_synced {
                    println!(
                        "Branch `{}` is up-to-date with the remote. Skipping push.",
                        Color::Green.paint(branch)
                    );
                    continue;
                }

                // Push the branch to the remote.
                ctx.repository.push_branch(branch, "origin", self.force)?;

                // Print success message.
                println!("Updated branch `{}` on remote.", Color::Green.paint(branch));
            } else {
                // If the PR has not been submitted yet.

                // Verify the base branch exists on remote
                let remote_base = format!("origin/{}", parent);
                if ctx.repository.find_branch(&remote_base, BranchType::Remote).is_err() {
                    return Err(StError::BaseBranchNotOnRemote(parent.clone()));
                }

                // Push the branch to the remote.
                ctx.repository.push_branch(branch, "origin", self.force)?;

                // Get the diff between the branch and its parent
                let diff = ctx
                    .repository
                    .diff_branches(branch, &parent)
                    .unwrap_or_else(|_| String::from("Unable to generate diff"));

                // Get commit messages between the branch and its parent
                let commits = ctx
                    .repository
                    .commit_messages_between(branch, &parent)
                    .unwrap_or_else(|_| vec![]);

                // Prompt the user for PR metadata.
                let metadata = Self::prompt_pr_metadata(&mut ctx.cfg, branch, &parent, &commits, &diff).await?;

                // Submit PR.
                let pr_info = pulls
                    .create(metadata.title, branch, &parent)
                    .body(metadata.body)
                    .draft(metadata.is_draft)
                    .send()
                    .await?;

                // Update the tracked branch with the remote information.
                tracked_branch.remote = Some(RemoteMetadata::new(pr_info.number));

                // Print success message.
                let pr_link = format!(
                    "https://github.com/{}/{}/pull/{}",
                    owner, repo, pr_info.number
                );
                println!(
                    "Submitted new pull request for branch `{}` @ `{}`",
                    Color::Green.paint(branch),
                    Color::Blue.paint(pr_link)
                );
            }
        }

        Ok(())
    }

    /// Updates the comments on a PR with the current stack information.
    async fn update_pr_comments(
        &self,
        ctx: &mut StContext<'_>,
        issue_handler: IssueHandler<'_>,
        stack: &[String],
    ) -> StResult<()> {
        for branch in stack.iter().skip(1) {
            let tracked_branch = ctx
                .tree
                .get_mut(branch)
                .ok_or_else(|| StError::BranchNotTracked(branch.to_string()))?;

            // Skip branches that are not submitted as PRs.
            let Some(remote_meta) = tracked_branch.remote else {
                continue;
            };

            // If the PR has been submitted, update the comment.
            // If the PR is new, create a new comment.
            let rendered_comment = Self::render_pr_comment(ctx, branch, stack)?;
            match remote_meta.comment_id {
                Some(id) => {
                    // Update the existing comment.
                    issue_handler
                        .update_comment(CommentId(id), rendered_comment)
                        .await?;
                }
                None => {
                    // Create a new comment.
                    let comment_info = issue_handler
                        .create_comment(remote_meta.pr_number, rendered_comment)
                        .await?;

                    // Get a new mutable reference to the branch and update the comment ID.
                    ctx.tree
                        .get_mut(branch)
                        .expect("Must exist")
                        .remote
                        .as_mut()
                        .expect("Must exist")
                        .comment_id = Some(comment_info.id.0);
                }
            }
        }
        Ok(())
    }

    /// Prompts the user for metadata about the PR during the initial submission process.
    async fn prompt_pr_metadata(
        config: &mut crate::config::StConfig,
        branch_name: &str,
        parent_name: &str,
        commits: &[String],
        diff: &str,
    ) -> StResult<PRCreationMetadata> {
        let title = inquire::Text::new(
            format!(
                "Title of pull request (`{}` -> `{}`):" ,
                Color::Green.paint(branch_name),
                Color::Yellow.paint(parent_name)
            )
            .as_str(),
        )
        .prompt()?;

        // Check if Ollama is available and offer AI generation
        let use_ai = if crate::ai::is_ollama_available().await {
            inquire::Confirm::new("Use AI to generate PR description?")
                .with_default(false)
                .prompt()
                .unwrap_or(false)
        } else {
            false
        };

        let body = if use_ai {
            // List available models
            let models = crate::ai::list_models().await?;
            if models.is_empty() {
                eprintln!(
                    "{}",
                    Color::Yellow.paint("No Ollama models found. Falling back to manual entry.")
                );
                inquire::Editor::new("Pull request description")
                    .with_file_extension(".md")
                    .prompt()?
            } else {
                // Check if saved model preference exists and is still available
                let model = if !config.ollama_model.is_empty() 
                    && models.contains(&config.ollama_model) {
                    println!(
                        "{} {}",
                        Color::Blue.paint("Using saved model:"),
                        Color::Green.paint(&config.ollama_model)
                    );
                    config.ollama_model.clone()
                } else {
                    // Ask user to select a model
                    let selected = inquire::Select::new("Select Ollama model:", models).prompt()?;
                    // Save the preference
                    config.ollama_model = selected.clone();
                    selected
                };

                println!(
                    "{}",
                    Color::Blue.paint("Generating PR description with AI...")
                );

                match crate::ai::generate_pr_description(&model, &title, branch_name, parent_name, commits, diff)
                    .await
                {
                    Ok(generated) => {
                        println!(
                            "{}",
                            Color::Green.paint("✓ Generated PR description. Review and edit if needed.")
                        );
                        // Let user review and edit the AI-generated description
                        inquire::Editor::new("Review and edit PR description")
                            .with_file_extension(".md")
                            .with_predefined_text(&generated)
                            .prompt()?
                    }
                    Err(e) => {
                        eprintln!(
                            "{}: {}",
                            Color::Red.paint("AI generation failed"),
                            e
                        );
                        inquire::Editor::new("Pull request description")
                            .with_file_extension(".md")
                            .prompt()?
                    }
                }
            }
        } else {
            inquire::Editor::new("Pull request description")
                .with_file_extension(".md")
                .prompt()?
        };

        let is_draft = inquire::Confirm::new("Is this PR a draft? (default: yes)")
            .with_default(true)
            .prompt()?;

        Ok(PRCreationMetadata {
            title,
            body,
            is_draft,
        })
    }

    /// Renders the PR comment body for the current stack.
    fn render_pr_comment(
        ctx: &StContext<'_>,
        current_branch: &str,
        stack: &[String],
    ) -> StResult<String> {
        let mut comment = String::new();
        comment.push_str("## 📚 $\\text{Stack Overview}$\n\n");
        comment.push_str("Pulls submitted in this stack:\n");

        // Display all branches in the stack.
        for branch in stack.iter().skip(1).rev() {
            let tracked_branch = ctx
                .tree
                .get(branch)
                .ok_or_else(|| StError::BranchNotTracked(branch.to_string()))?;
            if let Some(remote) = tracked_branch.remote {
                comment.push_str(&format!(
                    "* #{}{}\n",
                    remote.pr_number,
                    (branch == current_branch)
                        .then_some(" 👈")
                        .unwrap_or_default()
                ));
            }
        }
        comment.push_str(format!("* `{}`\n", ctx.tree.trunk_name()).as_str());

        comment.push_str(
            "\n_This comment was automatically generated by [`st`](https://github.com/mahbd/st)._",
        );
        Ok(comment)
    }
}

/// Metadata about pull request creation.
struct PRCreationMetadata {
    /// Title of the pull request.
    title: String,
    /// Body of the pull request.
    body: String,
    /// Whether or not the pull request is a draft.
    is_draft: bool,
}
